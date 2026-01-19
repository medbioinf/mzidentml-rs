use std::{
    borrow::Cow,
    collections::HashMap,
    ops::Deref,
    sync::{Arc, LazyLock},
};

use bincode::{Decode, Encode};
use context_error::{BoxedError, CreateError, FullErrorContent, StaticErrorContent};
use mzcv::{
    CVData, CVError, CVFile, CVIndex, CVSource, CVVersion, ControlledVocabulary, HashBufReader,
    OboOntology, OboStanzaType,
};

use crate::error::CvError;

/// Static error which can be reused without cloning
///
pub static UNKNOWN_CV_ERROR: CvError = CvError::UnknownCv;

/// MS CV where CVData includes children
///
pub static MS_CVINDEX: LazyLock<Result<WrappedCvIndex<MS>, CvError>> =
    LazyLock::new(WrappedCvIndex::init);

/// Unimod CV where CVData includes children
///
pub static UNIMOD_CVINDEX: LazyLock<Result<WrappedCvIndex<Unimod>, CvError>> =
    LazyLock::new(WrappedCvIndex::init);

/// Wrapper around CvIndex to ease accessing children.
///
pub struct WrappedCvIndex<T: CVSource<Data = CvDataWithChildren>>(CVIndex<T>);

impl<T: CVSource<Data = CvDataWithChildren>> WrappedCvIndex<T> {
    /// Initializes CV
    ///
    pub fn init() -> Result<WrappedCvIndex<T>, CvError> {
        let cv_index = if T::default_stem().with_extension("bin").is_file() {
            let (cv_index, init_errors) = CVIndex::<T>::init();

            if !init_errors.is_empty() {
                return Err(crate::error::CvError::IndexInit(
                    T::cv_name(),
                    Arc::new(init_errors),
                ));
            }

            cv_index
        } else {
            let mut cv_index = CVIndex::<T>::empty();

            cv_index
                .update_from_url(&[])
                .map_err(|err| crate::error::CvError::Download(T::cv_name(), Arc::new(err)))?;

            cv_index
                .save_to_cache()
                .map_err(|err| CvError::SaveToCache(T::cv_name(), Arc::new(err)))?;

            cv_index
        };

        Ok(WrappedCvIndex(cv_index))
    }

    /// Get children of the given term
    ///
    /// # Arguments
    /// * `term_id` - Numeric part of the term ID
    ///
    pub fn children_of(&self, term_id: &usize) -> Result<Vec<Arc<CvDataWithChildren>>, CvError> {
        let term = self
            .get_by_index(term_id)
            .ok_or_else(|| CvError::UnknownCvTerm(T::cv_name().to_string(), *term_id))?;

        self.inner_children_of(term.as_ref())
    }

    /// Recursive inner function to get children of children
    ///
    /// # Arguments
    /// * `term_id` - Numeric part of the term ID
    ///
    fn inner_children_of(
        &self,
        term: &CvDataWithChildren,
    ) -> Result<Vec<Arc<CvDataWithChildren>>, CvError> {
        let child_terms = term
            .children()
            .iter()
            .map(|child_idx| {
                self.get_by_index(child_idx).ok_or_else(|| {
                    CvError::UnknownCvTerm(T::cv_name().to_string(), term.index().unwrap())
                })
            })
            .collect::<Result<Vec<_>, CvError>>()?;

        child_terms
            .into_iter()
            .try_fold(Vec::new(), |mut terms, child_term| {
                terms.push(child_term);
                if !terms.last().unwrap().children().is_empty() {
                    terms.extend(self.inner_children_of(terms.last().unwrap().as_ref())?);
                }
                Ok(terms)
            })
    }
}

impl<T> Deref for WrappedCvIndex<T>
where
    T: CVSource<Data = CvDataWithChildren>,
{
    type Target = CVIndex<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::fmt::Debug for WrappedCvIndex<T>
where
    T: CVSource<Data = CvDataWithChildren>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WrappedCvIndex")
            .field("cv", &T::cv_name())
            .finish()
    }
}

/// Minimum represenation of CV terms for validation
///
#[derive(Clone, Debug, Decode, Default, Encode)]
pub struct CvDataWithChildren {
    /// Numeric part of the entry's accession code (e.g., 1000123 for MS:1000123)
    index: Option<usize>,
    /// Name
    name: Box<str>,
    /// Numeric part of the parent entries accession code
    is_a: Vec<usize>,
    /// Numeric part of the child entries accession code
    children: Vec<usize>,
}

impl CvDataWithChildren {
    fn children(&self) -> &[usize] {
        &self.children
    }
}

impl CVData for CvDataWithChildren {
    type Index = usize;
    fn index(&self) -> Option<usize> {
        self.index
    }
    fn curie(&self) -> Option<mzcv::Curie> {
        self.index
            .map(|v| ControlledVocabulary::MS.curie(mzcv::AccessionCode::Numeric(v as u32)))
    }
    fn name(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(&self.name))
    }

    /// Not used
    fn synonyms(&self) -> impl Iterator<Item = &str> {
        std::iter::empty()
    }

    fn parents(&self) -> impl Iterator<Item = &Self::Index> {
        self.is_a.iter()
    }
}

/// Parses a OBO file into CV with CvDataWithChildren
///
/// # Argument
/// * `reader` - File reader
///
#[allow(clippy::type_complexity)]
fn parse_obo_cv_with_children(
    mut reader: impl Iterator<Item = HashBufReader<Box<dyn std::io::Read>, impl sha2::Digest>>,
) -> Result<(CVVersion, Vec<Arc<CvDataWithChildren>>), Vec<BoxedError<'static, CVError>>> {
    let reader = reader.next().unwrap();
    OboOntology::from_raw(reader)
        .map_err(|e| {
            vec![
                BoxedError::small(
                    CVError::FileCouldNotBeParsed,
                    e.get_short_description(),
                    e.get_long_description(),
                )
                .add_contexts(e.get_contexts().iter().cloned()),
            ]
        })
        .map(|obo| {
            let version = obo.version();
            // Map to temporarily store childrem IDs while creating items
            let mut children_map: HashMap<usize, Vec<usize>> = HashMap::new();
            let mut ms_data_items: Vec<Arc<CvDataWithChildren>> = obo
                .objects
                .into_iter()
                .filter(|o| o.stanza_type == OboStanzaType::Term)
                .map(|obj| {
                    let mut data = CvDataWithChildren {
                        index: obj.id.1.parse().ok(),
                        name: obj.lines["name"][0].0.clone(),
                        ..Default::default()
                    };
                    for parent in obj.is_a.iter() {
                        if let Ok(parent_id) = parent.1.parse() {
                            data.is_a.push(parent_id);
                            if let Some(index) = data.index {
                                children_map.entry(parent_id).or_default().push(index);
                            }
                        }
                    }
                    Arc::new(data)
                })
                .collect();

            for item in ms_data_items.iter_mut() {
                if let Some(index) = item.index
                    && let Some(children) = children_map.get(&index)
                {
                    Arc::get_mut(item).unwrap().children = children.clone();
                }
            }

            (version, ms_data_items)
        })
}

/// Mass Spectrometry Ontology (MS)
///
pub struct MS;

impl CVSource for MS {
    type Data = CvDataWithChildren;
    type Structure = Vec<Arc<CvDataWithChildren>>;
    fn cv_name() -> &'static str {
        "MS4mzIdentMl" // TODO: Rename once this is not overriding other CVs in the cache folder
    }
    fn files() -> &'static [CVFile] {
        &[CVFile {
            name: "MS",
            extension: "obo",
            url: Some("http://purl.obolibrary.org/obo/ms.obo"),
            compression: mzcv::CVCompression::None,
        }]
    }
    fn static_data() -> Option<(CVVersion, Self::Structure)> {
        None // TODO
    }
    fn parse(
        reader: impl Iterator<Item = HashBufReader<Box<dyn std::io::Read>, impl sha2::Digest>>,
    ) -> Result<(CVVersion, Self::Structure), Vec<BoxedError<'static, CVError>>> {
        parse_obo_cv_with_children(reader)
    }
}

pub struct Unimod;

impl CVSource for Unimod {
    type Data = CvDataWithChildren;
    type Structure = Vec<Arc<CvDataWithChildren>>;
    fn cv_name() -> &'static str {
        "Unimod4mzIdentMl" // TODO: Rename once this is not overriding other CVs in the cache folder
    }
    fn files() -> &'static [CVFile] {
        &[CVFile {
            name: "UNIMOD",
            extension: "obo",
            url: Some("https://www.unimod.org/obo/unimod.obo"),
            compression: mzcv::CVCompression::None,
        }]
    }
    fn static_data() -> Option<(CVVersion, Self::Structure)> {
        None // TODO
    }
    fn parse(
        reader: impl Iterator<Item = HashBufReader<Box<dyn std::io::Read>, impl sha2::Digest>>,
    ) -> Result<(CVVersion, Self::Structure), Vec<BoxedError<'static, CVError>>> {
        parse_obo_cv_with_children(reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Checks if childs if a term are correctly returned
    #[test]
    fn test_children_of() {
        let ms_cv = MS_CVINDEX.as_ref().unwrap();

        let children = ms_cv.children_of(&1001456).unwrap();

        // Search a n-th level child, e.g. software > analysis software > ProteoWizard software > ProteoWizard msconvert
        let msconvert_term = children.iter().find(|term| term.index.unwrap() == 1002205);
        assert!(msconvert_term.is_some())

        // Not checking any numbers as this constantly change when the CV is updated
    }

    /// Just test the correct initilization
    ///
    #[test]
    fn test_init() {
        let ms_cv = MS_CVINDEX.as_ref();
        assert!(ms_cv.is_ok(), "{:?}", ms_cv.unwrap_err().clone());

        let unimod_cv = UNIMOD_CVINDEX.as_ref();
        assert!(unimod_cv.is_ok(), "{:?}", unimod_cv.expect_err("{:?}"));
    }
}
