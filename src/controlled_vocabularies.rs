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

pub static UNKNOWN_CV_ERROR: CvError = CvError::UnknownCv;

pub static MS_CVINDEX: LazyLock<Result<WrappedCvIndex<MS>, CvError>> = LazyLock::new(|| {
    let (cv_index, init_errors) = CVIndex::<MS>::init();
    if cv_index.is_empty() {
        return Err(crate::error::CvError::IndexInit(
            MS::cv_name(),
            Arc::new(init_errors),
        ));
    }
    Ok(WrappedCvIndex(cv_index))
});

pub static UNIMOD_CVINDEX: LazyLock<Result<WrappedCvIndex<Unimod>, CvError>> =
    LazyLock::new(|| {
        let (cv_index, init_errors) = CVIndex::<Unimod>::init();
        if cv_index.is_empty() {
            return Err(crate::error::CvError::IndexInit(
                Unimod::cv_name(),
                Arc::new(init_errors),
            ));
        }
        Ok(WrappedCvIndex(cv_index))
    });

pub struct WrappedCvIndex<T: CVSource<Data = CvDataWithChildren>>(CVIndex<T>);

impl<T: CVSource<Data = CvDataWithChildren>> WrappedCvIndex<T> {
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

// .and_then(|child_term| {
//     // Unwrap should be good here, as only terms with an ID can be found.
//     let next_level_child_terms = self.children_of(child_term.index.unwrap())?;
//     std::iter::once(child_term).chain(next_level_child_terms.into_iter())
// })

impl<T> Deref for WrappedCvIndex<T>
where
    T: CVSource<Data = CvDataWithChildren>,
{
    type Target = CVIndex<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Minimum representation of CV terms for validation
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

/// Mass Spectrometry Ontology (MS)
///
pub struct MS;

impl CVSource for MS {
    type Data = CvDataWithChildren;
    type Structure = Vec<Arc<CvDataWithChildren>>;
    fn cv_name() -> &'static str {
        "MS"
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
        mut reader: impl Iterator<Item = HashBufReader<Box<dyn std::io::Read>, impl sha2::Digest>>,
    ) -> Result<(CVVersion, Self::Structure), Vec<BoxedError<'static, CVError>>> {
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
                // Map to temporarily store children IDs while creating items
                let mut children_map: HashMap<usize, Vec<usize>> = HashMap::new();
                let mut ms_data_items: Vec<Arc<CvDataWithChildren>> = obo
                    .objects
                    .into_iter()
                    .filter(|o| o.stanza_type == OboStanzaType::Term)
                    .map(|obj| {
                        if ["alternate mass", "ambiguous residues"]
                            .contains(&&*obj.lines["name"][0].0.clone())
                        {
                            println!("{:?}", obj);
                        }

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
}

pub struct Unimod;

impl CVSource for Unimod {
    type Data = CvDataWithChildren;
    type Structure = Vec<Arc<CvDataWithChildren>>;
    fn cv_name() -> &'static str {
        "Unimod"
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
        mut reader: impl Iterator<Item = HashBufReader<Box<dyn std::io::Read>, impl sha2::Digest>>,
    ) -> Result<(CVVersion, Self::Structure), Vec<BoxedError<'static, CVError>>> {
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
                // Map to temporarily store children IDs while creating items
                let mut children_map: HashMap<usize, Vec<usize>> = HashMap::new();
                let mut ms_data_items: Vec<Arc<CvDataWithChildren>> = obo
                    .objects
                    .into_iter()
                    .filter(|o| o.stanza_type == OboStanzaType::Term)
                    .map(|obj| {
                        if ["alternate mass", "ambiguous residues"]
                            .contains(&&*obj.lines["name"][0].0.clone())
                        {
                            println!("{:?}", obj);
                        }

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
}

#[cfg(test)]
mod tests {
    use super::MS_CVINDEX;

    #[test]
    fn test_children_of() {
        let ms_cv = MS_CVINDEX.as_ref().unwrap();

        let children = ms_cv.children_of(&1001456).unwrap();

        // Search a nth level child, e.g. software > analysis software > ProteoWizard software > ProteoWizard msconvert
        let msconvert_term = children.iter().find(|term| term.index.unwrap() == 1002205);
        assert!(msconvert_term.is_some())

        // Not checking any numbers as this constantly change when the CV is updated
    }
}
