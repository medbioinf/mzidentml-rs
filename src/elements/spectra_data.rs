use serde::{Deserialize, Serialize};

use crate::elements::external_format_documentation::ExternalFormatDocumentation;
use crate::elements::file_format::FileFormat;
use crate::elements::is_element::IsElement;
use crate::elements::spectrum_id_format::SpectrumIDFormat;
use crate::error::ValidationError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpectraData {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@location")]
    pub location: String,

    #[serde(rename = "@name")]
    pub name: Option<String>,

    #[serde(rename = "ExternalFormatDocumentation")]
    pub external_format_documentation: Option<ExternalFormatDocumentation>,
    #[serde(rename = "FileFormat")]
    pub file_format: Option<FileFormat>,
    #[serde(rename = "SpectrumIDFormat")]
    pub spectrum_id_format: SpectrumIDFormat,
}

impl IsElement for SpectraData {
    fn validate(&self, strict: bool) -> Result<(), ValidationError> {
        if self.id.is_empty() {
            return Err(ValidationError::EmptyAttribute("SpectraData", "id"));
        }
        if self.location.is_empty() {
            return Err(ValidationError::EmptyAttribute("SpectraData", "location"));
        }

        if let Some(external_format_documentation) = &self.external_format_documentation {
            external_format_documentation.validate(strict)?;
        }
        if let Some(file_format) = &self.file_format {
            file_format.validate(strict)?;
        }

        self.spectrum_id_format.validate(strict)
    }
}
