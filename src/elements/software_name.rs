use serde::{Deserialize, Serialize};

use crate::{
    elements::{
        attributes::semver::SemVer, cv_param::CvParam, is_element::IsElement, user_param::UserParam,
    },
    error::ValidationError,
    has_cv_params,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoftwareName {
    #[serde(default, rename = "cvParam")]
    pub cv_params: Vec<CvParam>, // TODO: Actually only one is allowed, but has_cv_params! accepts only Vec fields now.
    #[serde(default, rename = "userParam")]
    pub user_params: Vec<UserParam>,
}

impl IsElement for SoftwareName {
    const ELEMENT_TAG: &str = "SoftwareName";

    fn inner_validate(
        &self,
        version: &SemVer,
        strict: bool,
        element_path: &mut Vec<String>,
    ) -> Result<(), ValidationError> {
        if self.cv_params.is_empty() {
            return Err(ValidationError::ChildRequiredAtLeastOnce(
                Self::element_path_to_string(element_path),
                "cvParam",
            ));
        }
        self.validate_cv_params(version, strict, element_path)?;

        if !self.cv_param_by_accession("MS", 1000799)?.is_empty() {
            return Err(ValidationError::ReasonedChildRequiredAtLeastOnce(
                Self::element_path_to_string(element_path),
                "userName",
                "MS:1000799 a userParam with the name of the software is needed.",
            ));
        }

        Self::validate_elements(version, strict, element_path, self.user_params.iter())
    }
}

has_cv_params!(
    SoftwareName,
    cv_params,
    [CvParamRule {
        cv_name: "MS",
        id: 1001456,
        occurence: CvParamOccurence::MustOnceOrMany,
        supplies_children: true,
    },]
);
