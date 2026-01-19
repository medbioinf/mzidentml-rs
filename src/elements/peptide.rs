use serde::{Deserialize, Serialize};

use crate::{
    elements::{
        attributes::semver::SemVer, cv_param::CvParam, is_element::IsElement,
        modification::Modification, peptide_sequence::PeptideSequence,
        substitution_modification::SubstitutionModification, user_param::UserParam,
    },
    error::ValidationError,
    has_cv_params,
    indexed_elements::is_indexed_element::IsIndexedSubelement,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Peptide {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "PeptideSequence")]
    pub peptide_sequence: PeptideSequence,
    #[serde(default, rename = "Modification")]
    pub modifications: Vec<Modification>,
    #[serde(default, rename = "SubstitutionModification")]
    pub substitution_modifications: Vec<SubstitutionModification>,
    #[serde(default, rename = "cvParam")]
    pub cv_params: Vec<CvParam>,
    #[serde(default, rename = "userParam")]
    pub user_params: Vec<UserParam>,
}

impl IsElement for Peptide {
    const ELEMENT_TAG: &str = "Peptide";

    fn inner_validate(
        &self,
        version: &SemVer,
        strict: bool,
        element_path: &mut Vec<String>,
    ) -> Result<(), ValidationError> {
        if self.id.is_empty() {
            return Err(ValidationError::EmptyAttribute(
                Self::element_path_to_string(element_path),
                "id",
            ));
        }

        self.peptide_sequence
            .validate(version, strict, element_path, None)?;

        Self::validate_elements(version, strict, element_path, self.modifications.iter())?;

        Self::validate_elements(
            version,
            strict,
            element_path,
            self.substitution_modifications.iter(),
        )?;

        self.validate_cv_params(version, strict, element_path)?;
        Self::validate_elements(version, strict, element_path, self.user_params.iter())
    }
}

has_cv_params!(
    Peptide,
    cv_params,
    [CvParamRule {
        cv_name: "MS",
        id: 1001355,
        occurence: CvParamOccurence::MayOnceOrMany,
        supplies_children: true,
    },]
);

impl IsIndexedSubelement for Peptide {
    fn identifier(&self) -> &str {
        &self.id
    }
}
