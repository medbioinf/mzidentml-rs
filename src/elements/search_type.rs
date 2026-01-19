use serde::{Deserialize, Serialize};

use crate::{
    elements::{
        attributes::semver::SemVer, cv_param::CvParam, is_element::IsElement, user_param::UserParam,
    },
    error::ValidationError,
    has_cv_params,
};

// TODO: This might not be fully correct. SearchType is supposed to have 1 CvParam and 1 UserParam. It might miss similar to SoftwareName.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchType {
    #[serde(default, rename = "cvParam")]
    pub cv_params: Vec<CvParam>,
    #[serde(default, rename = "userParam")]
    pub user_params: Vec<UserParam>,
}

impl IsElement for SearchType {
    const ELEMENT_TAG: &str = "SearchType";

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

        Self::validate_elements(version, strict, element_path, self.user_params.iter())
    }
}

has_cv_params!(
    SearchType,
    cv_params,
    [CvParamRule {
        cv_name: "MS",
        id: 1001080,
        occurence: CvParamOccurence::MustOnceOrMany,
        supplies_children: true,
    },]
);
