use serde::{Deserialize, Serialize};

use crate::{
    elements::{
        attributes::semver::SemVer, cv_param::CvParam, is_element::IsElement,
        specificity_rules::SpecificityRules,
    },
    error::ValidationError,
    has_cv_params,
    parsing::space_separated_vec_parsing,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchModification {
    #[serde(rename = "@fixedMod")]
    pub fixed_mod: bool,
    #[serde(rename = "@massDelta")]
    pub mass_delta: f64,
    #[serde(rename = "@residues", with = "space_separated_vec_parsing")]
    pub residues: Vec<char>,

    #[serde(default, rename = "cvParam")]
    pub cv_params: Vec<CvParam>,
    #[serde(default, rename = "SpecificityRules")]
    pub specificity_rules: Vec<SpecificityRules>,
}

// TODO: [Definition](https://raw.githubusercontent.com/HUPO-PSI/mzIdentML/2aacf89e164afc96f71dee7e433c055718d7db0d/specification_document-releases/specdoc1_3/mzIdentML1.3.0-release.pdf#%5B%7B%22num%22%3A214%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C192.2%2C692.4%2C0%5D)
// `MAY MS:1003392` needs to be validated accordingly over the complete document
impl IsElement for SearchModification {
    const ELEMENT_TAG: &str = "SearchModification";

    fn inner_validate(
        &self,
        version: &SemVer,
        strict: bool,
        element_path: &mut Vec<String>,
    ) -> Result<(), ValidationError> {
        if self.residues.is_empty() {
            return Err(ValidationError::EmptyAttribute(
                Self::element_path_to_string(element_path),
                "residues",
            ));
        }

        if self.cv_params.is_empty() {
            return Err(ValidationError::ChildRequiredAtLeastOnce(
                Self::element_path_to_string(element_path),
                "cvParam",
            ));
        }

        self.validate_cv_params(version, strict, element_path)?;

        Self::validate_elements(version, strict, element_path, self.specificity_rules.iter())
    }
}

has_cv_params!(
    SearchModification,
    cv_params,
    [
        CvParamRule {
            cv_name: "MS",
            id: 1001460,
            occurence: CvParamOccurence::MustOnce,
            supplies_children: false,
        },
        CvParamRule {
            cv_name: "MS",
            id: 1003392,
            occurence: CvParamOccurence::MayOnce,
            supplies_children: false,
        },
        CvParamRule {
            cv_name: "MS",
            id: 1002509,
            occurence: CvParamOccurence::MayOnce,
            supplies_children: false,
        },
        CvParamRule {
            cv_name: "MS",
            id: 1002510,
            occurence: CvParamOccurence::MayOnce,
            supplies_children: false,
        },
        CvParamRule {
            cv_name: "MS",
            id: 1002504,
            occurence: CvParamOccurence::MayOnce,
            supplies_children: false,
        },
        CvParamRule {
            cv_name: "UNIMOD",
            id: 0,
            occurence: CvParamOccurence::MayOnce,
            supplies_children: true,
        },
        // TODO: XLMOD & Mod needs to be implemented
        // CvParamRule {
        //     cv_name: "XLMOD",
        //     id: 2,
        //     occurence: CvParamOccurence::MayOnce,
        //     supplies_children: true,
        // },
        // CvParamRule {
        //     cv_name: "XLMOD",
        //     id: 4,
        //     occurence: CvParamOccurence::MayOnce,
        //     supplies_children: true,
        // },
        // CvParamRule {
        //     cv_name: "Mod",
        //     id: 0,
        //     occurence: CvParamOccurence::MayOnce,
        //     supplies_children: true,
        // },
        CvParamRule {
            cv_name: "MS",
            id: 1001471,
            occurence: CvParamOccurence::MayOnceOrMany,
            supplies_children: true,
        },
    ]
);
