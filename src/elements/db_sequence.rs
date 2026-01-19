use serde::{Deserialize, Serialize};

use crate::{
    elements::{
        attributes::semver::SemVer, cv_param::CvParam, is_element::IsElement, seq::Seq,
        user_param::UserParam,
    },
    error::ValidationError,
    has_cv_params,
    indexed_elements::is_indexed_element::IsIndexedSubelement,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbSequence {
    #[serde(rename = "@accession")]
    pub accession: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@searchDatabase_ref")]
    pub search_database_ref: String,

    #[serde(rename = "@length")]
    pub length: Option<usize>,
    #[serde(rename = "@name")]
    pub name: Option<String>,

    #[serde(rename = "Seq")]
    pub sequence: Option<Seq>,
    #[serde(default, rename = "cvParam")]
    pub cv_params: Vec<CvParam>,
    #[serde(default, rename = "userParam")]
    pub user_params: Vec<UserParam>,
}

impl IsElement for DbSequence {
    const ELEMENT_TAG: &str = "DbSequence";

    fn inner_validate(
        &self,
        version: &SemVer,
        strict: bool,
        element_path: &mut Vec<String>,
    ) -> Result<(), ValidationError> {
        if self.accession.is_empty() {
            return Err(ValidationError::EmptyAttribute(
                Self::element_path_to_string(element_path),
                "accession",
            ));
        }

        if self.id.is_empty() {
            return Err(ValidationError::EmptyAttribute(
                Self::element_path_to_string(element_path),
                "id",
            ));
        }

        if self.search_database_ref.is_empty() {
            return Err(ValidationError::EmptyAttribute(
                Self::element_path_to_string(element_path),
                "searchDatabase_ref",
            ));
        }

        self.validate_cv_params(version, strict, element_path)?;

        Self::validate_elements(version, strict, element_path, self.user_params.iter())?;

        if let Some(seq) = &self.sequence {
            seq.validate(version, strict, element_path, None)?;
        }

        Ok(())
    }
}

has_cv_params!(
    DbSequence,
    cv_params,
    [
        CvParamRule {
            cv_name: "MS",
            id: 1001342,
            occurence: CvParamOccurence::MayOnceOrMany,
            supplies_children: true,
        },
        CvParamRule {
            cv_name: "MS",
            id: 1001089,
            occurence: CvParamOccurence::MayOnceOrMany,
            supplies_children: true,
        }
    ]
);

impl IsIndexedSubelement for DbSequence {
    fn identifier(&self) -> &str {
        &self.id
    }
}
