use serde::{Deserialize, Serialize};

use crate::{
    elements::{
        ALLOWED_SUBSTITUTION_RESIDUES, attributes::semver::SemVer, cv_param::CvParam,
        is_element::IsElement, user_param::UserParam,
    },
    error::ValidationError,
    has_cv_params,
    indexed_elements::is_indexed_element::IsIndexedSubelement,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeptideEvidence {
    #[serde(rename = "@dBSequence_ref")]
    pub db_sequence_ref: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@peptide_ref")]
    pub peptide_ref: String,
    #[serde(rename = "@end")]
    pub end: Option<usize>,
    #[serde(rename = "@frame")]
    pub frame: Option<String>, // TODO: Implement datatype allowed_frames as stated in specs
    #[serde(rename = "@isDecoy")]
    pub is_decoy: Option<bool>,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@post")]
    pub post: Option<char>,
    #[serde(rename = "@pre")]
    pub pre: Option<char>,
    #[serde(rename = "@start")]
    pub start: Option<usize>,
    #[serde(rename = "@translationTable_ref")]
    pub translation_table_ref: Option<String>,
    #[serde(default, rename = "cvParam")]
    pub cv_params: Vec<CvParam>,
    #[serde(default, rename = "userParam")]
    pub user_params: Vec<UserParam>,
}

impl IsElement for PeptideEvidence {
    const ELEMENT_TAG: &str = "PeptideEvidence";

    fn inner_validate(
        &self,
        version: &SemVer,
        strict: bool,
        element_path: &mut Vec<String>,
    ) -> Result<(), ValidationError> {
        if self.db_sequence_ref.is_empty() {
            return Err(ValidationError::EmptyAttribute(
                Self::element_path_to_string(element_path),
                "peptide_ref",
            ));
        }
        if self.id.is_empty() {
            return Err(ValidationError::EmptyAttribute(
                Self::element_path_to_string(element_path),
                "id",
            ));
        }

        if let Some(post) = &self.post
            && !ALLOWED_SUBSTITUTION_RESIDUES.contains(post)
        {
            return Err(ValidationError::InvalidAttributeValue(
                "PeptideEvidence",
                "post",
                ALLOWED_SUBSTITUTION_RESIDUES
                    .iter()
                    .map(|res| res.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }

        if let Some(pre) = &self.pre
            && !ALLOWED_SUBSTITUTION_RESIDUES.contains(pre)
        {
            return Err(ValidationError::InvalidAttributeValue(
                "PeptideEvidence",
                "pre",
                ALLOWED_SUBSTITUTION_RESIDUES
                    .iter()
                    .map(|res| res.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }

        self.validate_cv_params(version, strict, element_path)?;

        Self::validate_elements(version, strict, element_path, self.user_params.iter())
    }
}

has_cv_params!(PeptideEvidence, cv_params);

impl IsIndexedSubelement for PeptideEvidence {
    fn identifier(&self) -> &str {
        &self.id
    }
}
