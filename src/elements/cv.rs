use serde::{Deserialize, Serialize};

use crate::{elements::attributes::semver::SemVer, error::ValidationError};

use super::is_element::IsElement;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cv {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@fullName")]
    pub full_name: String,
    #[serde(rename = "@version")]
    pub version: Option<String>, // TODO: Implement semver like struct
    #[serde(rename = "@uri")]
    pub uri: String, // TODO: Proper URI check
}

impl IsElement for Cv {
    const ELEMENT_TAG: &str = "cv";

    fn inner_validate(
        &self,
        _version: &SemVer,
        _strict: bool,
        element_path: &mut Vec<String>,
    ) -> Result<(), ValidationError> {
        if self.id.is_empty() {
            return Err(ValidationError::EmptyAttribute(
                Self::element_path_to_string(element_path),
                "id]",
            ));
        }

        if self.full_name.is_empty() {
            return Err(ValidationError::EmptyAttribute(
                Self::element_path_to_string(element_path),
                "full_name",
            ));
        }

        if self.uri.is_empty() {
            return Err(ValidationError::EmptyAttribute(
                Self::element_path_to_string(element_path),
                "uri",
            ));
        }

        Ok(())
    }
}
