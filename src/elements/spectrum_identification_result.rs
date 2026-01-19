use serde::{Deserialize, Serialize};

use crate::{
    elements::{
        attributes::semver::SemVer, cv_param::CvParam, is_element::IsElement,
        spectrum_identification_item::SpectrumIdentificationItem, user_param::UserParam,
    },
    error::ValidationError,
    has_cv_params,
    indexed_elements::is_indexed_element::IsIndexedSubelement,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpectrumIdentificationResult {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@spectraData_ref")]
    pub spectra_data_ref: String,
    #[serde(rename = "@spectrumID")]
    pub spectrum_id: String,

    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "SpectrumIdentificationItem")]
    pub spectrum_identification_items: Vec<SpectrumIdentificationItem>,

    #[serde(default, rename = "cvParam")]
    pub cv_params: Vec<CvParam>,

    #[serde(default, rename = "userParam")]
    pub user_params: Vec<UserParam>,
}

impl IsElement for SpectrumIdentificationResult {
    const ELEMENT_TAG: &str = "SpectrumIdentificationResult";

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

        if self.spectra_data_ref.is_empty() {
            return Err(ValidationError::EmptyAttribute(
                Self::element_path_to_string(element_path),
                "spectraData_ref",
            ));
        }

        if self.spectrum_identification_items.is_empty() {
            return Err(ValidationError::EmptyAttribute(
                Self::element_path_to_string(element_path),
                "SpectrumIdentificationItem",
            ));
        }

        Self::validate_elements(
            version,
            strict,
            element_path,
            self.spectrum_identification_items.iter(),
        )?;

        self.validate_cv_params(version, strict, element_path)?;

        Self::validate_elements(version, strict, element_path, self.user_params.iter())
    }
}

has_cv_params!(
    SpectrumIdentificationResult,
    cv_params,
    [CvParamRule {
        cv_name: "MS",
        id: 1001405,
        occurence: CvParamOccurence::MayOnceOrMany,
        supplies_children: true
    }]
);

impl IsIndexedSubelement for SpectrumIdentificationResult {
    fn identifier(&self) -> &str {
        &self.id
    }
}
