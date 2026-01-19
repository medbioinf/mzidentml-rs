use serde::{Deserialize, Serialize};

use crate::{
    elements::{attributes::semver::SemVer, cv_param::CvParam, is_element::IsElement},
    error::ValidationError,
    has_cv_params,
};

// TODO: Exactly the same as fragment tolerance, merge into one
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParentTolerance {
    #[serde(rename = "cvParam")]
    pub cv_params: Vec<CvParam>,
}

impl IsElement for ParentTolerance {
    const ELEMENT_TAG: &str = "ParentTolerance";

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

        self.validate_cv_params(version, strict, element_path)
    }
}

has_cv_params!(
    ParentTolerance,
    cv_params,
    [
        CvParamRule {
            cv_name: "MS",
            id: 1001412,
            occurence: CvParamOccurence::MustOnce,
            supplies_children: false,
        },
        CvParamRule {
            cv_name: "MS",
            id: 1001413,
            occurence: CvParamOccurence::MustOnce,
            supplies_children: false,
        }
    ]
);
