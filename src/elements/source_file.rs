use serde::{Deserialize, Serialize};

use crate::{
    elements::{
        cv_param::CvParam, external_format_documentation::ExternalFormatDocumentation,
        file_format::FileFormat, is_element::IsElement, user_param::UserParam,
    },
    error::ValidationError,
    has_cv_params,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceFile {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@location")]
    location: String,

    #[serde(rename = "@name")]
    name: Option<String>,

    #[serde(rename = "ExternalFormatDocumentation")]
    external_format_documentation: Option<ExternalFormatDocumentation>,
    #[serde(rename = "FileFormat")]
    file_format: Option<FileFormat>,
    #[serde(rename = "cvParam", default)]
    pub cv_params: Vec<CvParam>,
    #[serde(rename = "userParam", default)]
    pub user_params: Vec<UserParam>,
}

impl IsElement for SourceFile {
    fn validate(&self, strict: bool) -> Result<(), ValidationError> {
        if self.id.is_empty() {
            return Err(ValidationError::EmptyAttribute("SourceFile", "id"));
        }
        if self.location.is_empty() {
            return Err(ValidationError::EmptyAttribute("SourceFile", "location"));
        }

        if let Some(external_format_documentation) = &self.external_format_documentation {
            external_format_documentation.validate(strict)?;
        }
        if let Some(file_format) = &self.file_format {
            file_format.validate(strict)?;
        }

        self.validate_cv_params(strict)?;

        for param in self.user_params.iter() {
            param.validate(strict)?;
        }

        Ok(())
    }
}

has_cv_params!(
    SourceFile,
    cv_params,
    [CvParamRule {
        cv_name: "MS",
        id: 1000561,
        occurence: CvParamOccurence::MayOnceOrMany,
        supplies_children: true,
    },]
);
