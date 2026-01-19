use crate::{elements::attributes::semver::SemVer, error::ValidationError};

/// Trait to define common element behaviour. Most prominent validation
pub trait IsElement {
    /// Name of the elements tag in the mzIdentML
    const ELEMENT_TAG: &str;

    /// Validates the MzML element.
    /// Add the current element to the path than and than call [inner_validate]
    ///
    /// # Arguments
    /// * `version`- Document version
    /// * `strict` - If true, perform strict validation
    /// * `element_path` - Path to the current element in the document tree, with the calling element at the end.
    /// * `element_index` - Optional index of the element if it is part of a list
    ///
    fn validate(
        &self,
        version: &SemVer,
        strict: bool,
        element_path: &mut Vec<String>,
        element_index: Option<usize>,
    ) -> Result<(), ValidationError> {
        if let Some(index) = element_index {
            element_path.push(format!("{}[{index}]", Self::ELEMENT_TAG));
        } else {
            element_path.push(Self::ELEMENT_TAG.to_string());
        };
        self.inner_validate(version, strict, element_path)?;
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
    fn inner_validate(
        &self,
        version: &SemVer,
        strict: bool,
        element_path: &mut Vec<String>,
    ) -> Result<(), ValidationError>;

    /// Validated a couple of (sub) elements
    ///
    /// * `version`- Document version
    /// * `strict` - If true, perform strict validation
    /// * `element_path` - Path to the current element in the document tree with the current element at the end.
    /// * `elements` - (Sub) elements to validate
    ///
    fn validate_elements<'a, E: IsElement + 'a, I: Iterator<Item = &'a E>>(
        version: &SemVer,
        strict: bool,
        element_path: &mut Vec<String>,
        elements: I,
    ) -> Result<(), ValidationError> {
        for (element_idx, element) in elements.enumerate() {
            element.validate(version, strict, element_path, Some(element_idx))?;
        }
        Ok(())
    }

    /// Just joins the string vector, making sure to use the same separator each time.
    ///
    /// # Arguments
    /// * `element_path` - Path to the current element in the document tree with the current element at the end.
    ///
    fn element_path_to_string(element_path: &[String]) -> String {
        element_path_to_string(element_path)
    }
}

/// Just joins the string vector, making sure to use the same separator each time.
///
/// # Arguments
/// * `element_path` - Path to the current element in the document tree with the current element at the end.
///
pub fn element_path_to_string(element_path: &[String]) -> String {
    element_path.join(".")
}
