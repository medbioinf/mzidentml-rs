use serde::{Deserialize, Serialize};

use crate::{
    elements::{is_element::IsElement, organization::Organization, person::Person},
    error::ValidationError,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditCollection {
    #[serde(rename = "Person", default)]
    pub persons: Vec<Person>,
    #[serde(rename = "Organization")]
    pub organizations: Vec<Organization>,
}

impl IsElement for AuditCollection {
    fn validate(&self, strict: bool) -> Result<(), ValidationError> {
        if self.persons.is_empty() {
            return Err(ValidationError::ChildRequiredOnce(
                "AuditCollection",
                "Person",
            ));
        }

        for person in &self.persons {
            person.validate(strict)?;
        }

        if self.organizations.is_empty() {
            return Err(ValidationError::ChildRequiredOnce(
                "AuditCollection",
                "Organization",
            ));
        }

        for organization in &self.organizations {
            organization.validate(strict)?;
        }
        Ok(())
    }
}
