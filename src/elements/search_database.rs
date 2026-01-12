use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::elements::database_name::DatabaseName;
use crate::elements::external_format_documentation::ExternalFormatDocumentation;
use crate::elements::file_format::FileFormat;
use crate::error::ValidationError;
use crate::parsing::opt_date_time_parsing;
use crate::{
    elements::{cv_param::CvParam, is_element::IsElement},
    has_cv_params,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchDatabase {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@location")]
    pub location: String,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@numDatabaseSequences")]
    pub num_database_sequences: Option<usize>,
    #[serde(rename = "@numResidues")]
    pub num_residues: Option<usize>,
    #[serde(default, rename = "@releaseDate", with = "opt_date_time_parsing")]
    pub release_date: Option<DateTime<FixedOffset>>,
    #[serde(rename = "ExternalFormatDocumentation")]
    pub external_format_documentation: Option<ExternalFormatDocumentation>,
    #[serde(rename = "FileFormat")]
    pub file_format: Option<FileFormat>,
    #[serde(rename = "DatabaseName")]
    pub database_name: DatabaseName,
    #[serde(default, rename = "cvParam")]
    pub cv_params: Vec<CvParam>,
}

impl IsElement for SearchDatabase {
    fn validate(&self, strict: bool) -> Result<(), ValidationError> {
        if self.id.is_empty() {
            return Err(ValidationError::EmptyAttribute("SearchDatabase", "id"));
        }
        if self.location.is_empty() {
            return Err(ValidationError::EmptyAttribute(
                "SearchDatabase",
                "location",
            ));
        }

        if let Some(external_format_documentation) = &self.external_format_documentation {
            external_format_documentation.validate(strict)?;
        }
        if let Some(file_format) = &self.file_format {
            file_format.validate(strict)?;
        }

        self.database_name.validate(strict)?;

        self.validate_cv_params(strict)
    }
}

has_cv_params!(
    SearchDatabase,
    cv_params,
    [
        CvParamRule {
            cv_name: "MS",
            id: 1000561,
            occurence: CvParamOccurence::MayOnceOrMany,
            supplies_children: true,
        },
        CvParamRule {
            cv_name: "MS",
            id: 1001011,
            occurence: CvParamOccurence::MayOnceOrMany,
            supplies_children: true,
        },
    ]
);
