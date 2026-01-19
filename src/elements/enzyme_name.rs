use serde::{Deserialize, Serialize};

use crate::{
    elements::{
        attributes::semver::SemVer, cv_param::CvParam, is_element::IsElement, user_param::UserParam,
    },
    error::ValidationError,
    has_cv_params,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnzymeName {
    #[serde(default, rename = "cvParam")]
    pub cv_params: Vec<CvParam>,
    #[serde(default, rename = "userParam")]
    pub user_params: Vec<UserParam>,
}

impl IsElement for EnzymeName {
    const ELEMENT_TAG: &str = "EnzymeName";

    fn inner_validate(
        &self,
        version: &SemVer,
        strict: bool,
        element_path: &mut Vec<String>,
    ) -> Result<(), ValidationError> {
        // Validation is differ from specs document at the time of writing
        // https://github.com/HUPO-PSI/mzIdentML/issues/149#issuecomment-3761156439
        if self.cv_params.is_empty() && self.user_params.is_empty() {
            return Err(ValidationError::MissingChild(
                Self::element_path_to_string(element_path),
                "cvParam|userParam",
            ));
        }

        self.validate_cv_params(version, strict, element_path)?;

        Self::validate_elements(version, strict, element_path, self.user_params.iter())?;
        Ok(())
    }
}

has_cv_params!(
    EnzymeName,
    cv_params,
    [CvParamRule {
        cv_name: "MS",
        id: 1001045,
        occurence: CvParamOccurence::MayOnceOrMany,
        supplies_children: true,
    },]
);
