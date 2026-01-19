use std::io::{BufRead, Seek};

use crate::{
    elements::{attributes::semver::SemVer, is_element::IsElement},
    error::ValidationError,
};

/// Trait to define common element behaviour. Most prominent validation
pub trait IsIndexedElement: IsElement {
    /// Validates the MzML element.
    /// Add the current element to the path than and than call [inner_validate]
    ///
    /// # Arguments
    /// * `version`- Document version
    /// * `strict` - If true, perform strict validation
    /// * `element_path` - Path to the current element in the document tree, with the calling element at the end.
    /// * `element_index` - Optional index of the element if it is part of a list
    ///
    fn validate_indexed<R: BufRead + Seek>(
        &self,
        version: &SemVer,
        strict: bool,
        element_path: &mut Vec<String>,
        element_index: Option<usize>,
        reader: &mut R,
    ) -> Result<(), ValidationError> {
        if let Some(index) = element_index {
            element_path.push(format!("{}[{index}]", Self::ELEMENT_TAG));
        } else {
            element_path.push(Self::ELEMENT_TAG.to_string());
        };
        self.inner_validate_indexed(version, strict, element_path, reader)?;
        element_path.pop();
        Ok(())
    }

    /// Inner actual validation with the ready to use element path for this element
    ///
    /// # Arguments
    /// * `version`- Document version
    /// * `strict` - If true, perform strict validation
    /// * `element_path` - Path to the current element in the document tree with the current element at the end.
    ///
    fn inner_validate_indexed<R: BufRead + Seek>(
        &self,
        version: &SemVer,
        strict: bool,
        element_path: &mut Vec<String>,
        reader: &mut R,
    ) -> Result<(), ValidationError>;
}
