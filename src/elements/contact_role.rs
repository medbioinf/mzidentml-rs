use serde::{Deserialize, Serialize};

use crate::{
    elements::{is_element::IsElement, role::Role},
    error::ValidationError,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContactRole {
    #[serde(rename = "@contact_ref")]
    pub contact_ref: String,
    pub role: Option<Role>,
}

impl IsElement for ContactRole {
    fn validate(&self, strict: bool) -> Result<(), ValidationError> {
        if let Some(role) = &self.role {
            role.validate(strict)?;
        }
        Ok(())
    }
}
