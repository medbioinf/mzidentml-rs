use serde::{Deserialize, Serialize};

use crate::{
    elements::{attributes::semver::SemVer, is_element::IsElement, role::Role},
    error::ValidationError,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContactRole {
    #[serde(rename = "@contact_ref")]
    pub contact_ref: String,
    #[serde(rename = "Role")]
    pub role: Option<Role>,
}

impl IsElement for ContactRole {
    const ELEMENT_TAG: &str = "ContactRole";

    fn inner_validate(
        &self,
        version: &SemVer,
        strict: bool,
        element_path: &mut Vec<String>,
    ) -> Result<(), ValidationError> {
        if let Some(role) = self.role.as_ref() {
            role.validate(version, strict, element_path, None)?;
        }
        Ok(())
    }
}
