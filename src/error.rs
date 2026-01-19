use std::{fmt::Display, sync::Arc};

use thiserror::Error;

use crate::elements::has_cv_params::{CvParamOccurence, CvParamRule};

/// Things which can got wrong working with mzIdentML files.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Error `{}` at element MzIdentML.{}", .0.inner(), .0.path().to_string().trim_start_matches('.'))]
    Deserialization(#[from] serde_path_to_error::Error<quick_xml::DeError>),
    #[error("{0}")]
    Cv(#[from] CvError),
    #[error("{0}")]
    Validation(#[from] ValidationError),
}

#[derive(Clone, Debug, Error)]
pub enum ValidationError {
    #[error("Wrong cvParam rule, got {0} expected {1}")]
    WrongCvParamOccurence(CvParamOccurence, CvParamOccurence),
    #[error("{0}")]
    CvParamViolation(#[from] CvParamsValidationError),
    #[error("{0} > {1} is required at least once")]
    ChildRequiredOnce(&'static str, &'static str),
    #[error("{0} > {1} is required at least once")]
    ChildRequiredAtLeastOnce(String, &'static str),
    #[error("{0} > {1} is required at least once, due to {2}")]
    ReasonedChildRequiredAtLeastOnce(String, &'static str, &'static str),
    #[error("Missing element {0}")]
    MissingElement(&'static str),
    #[error("{0}[{1}] cannot be empty")]
    EmptyAttribute(String, &'static str),
    #[error("{0} > ({1:?}) is allowed at a time")]
    ExclisiveAttribute(String, &'static [&'static str]),
    #[error("{0}[{1}] has invalid expected `{2}`")]
    InvalidAttributeValue(&'static str, &'static str, String),
    #[error("Unable to parse version {0}, expected `major.minor.patch`")]
    InvalidVersion(String),
    #[error("{0}")]
    Cv(#[from] CvError),
    #[error("{0} > {1} is missing")]
    MissingChild(String, &'static str),
    #[error("{0}")]
    IndexedRead(#[from] ReadIndexedError),
}

/// Error for violated CvParam rules
#[derive(Clone, Debug, Error)]
pub enum CvParamsValidationError {
    RuleViolation(String, &'static CvParamRule, Option<Vec<String>>),
    Duplication(String, String, usize),
}

impl Display for CvParamsValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CvParamsValidationError::RuleViolation(element_path, rule, matching_accessions) => {
                let found = if let Some(matching_accessions) = matching_accessions {
                    matching_accessions.join(", ")
                } else {
                    "none".to_string()
                };
                write!(f, "{element_path}: Violated rule `{rule}`. Found {found}.")
            }
            CvParamsValidationError::Duplication(element_path, cv_name, term_id) => {
                write!(f, "{element_path}: Found duplicate for {cv_name}:{term_id}")
            }
        }
    }
}

#[derive(Clone, Debug, Error)]
pub enum CvError {
    #[error("Invalid CV accession format for `{0}`, expected `<ONTHOLOGY>:<ID>`")]
    InvalidIdFormat(String),
    #[error("Got CV `{0}` but could not parse second part of ID to integer `{1}`")]
    InvalidSecondPart(String, String),
    #[error("Unable to initialite CV index for `{0}`, because of the following reasons: {1:?}")]
    IndexInit(
        &'static str,
        Arc<Vec<context_error::BoxedError<'static, mzcv::CVError>>>,
    ),
    #[error("Unable to save CV `{0}` to cache, because of the following reasons: {1:?}")]
    SaveToCache(
        &'static str,
        Arc<context_error::BoxedError<'static, mzcv::CVError>>,
    ),
    #[error(
        "Unable to initialize CV `{0}` from inline ressource, because of the following reasons: {1:?}"
    )]
    Download(
        &'static str,
        Arc<context_error::BoxedError<'static, mzcv::CVError>>,
    ),
    #[error("Unknown CV")]
    UnknownCv,
    #[error("Unknown CV term: `{0}:{1}`. Maybe the CV source is outdated?")]
    UnknownCvTerm(String, usize),
}

#[derive(Debug, Error)]
pub enum IndexingError {
    #[error("Unable to move cursor to position `{0}`: {1}")]
    MoveCursor(usize, std::io::Error),
    #[error("Unable to find ID in element at position")]
    NoIdAttribute(u64),
    #[error("Unable to find end of ID attribute in element at position {0}")]
    IdAttributeNotClosed(u64),
    #[error("Unable to process XML {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("Validation error {0}")]
    Validation(#[from] ValidationError),
    #[error("Unable to process attribute: {0}")]
    Attribute(#[from] quick_xml::events::attributes::AttrError),
    #[error("{0}: Got {1} tag event, expected {2} tag.")]
    InvalidEvent(String, &'static str, &'static str),
    #[error("{0}: Got {1} tag, expected {2} tag.")]
    InvalidTag(String, String, &'static str),
    #[error("{0}: Unexpected EOF while looking for {1}")]
    UnexpectedEOF(String, String),
    #[error("{0}: {1}")]
    Deserialization(String, quick_xml::DeError),
}

#[derive(Clone, Debug, Error)]
pub enum ReadIndexedError {
    #[error("Unknown identifier")]
    UnknownIdentifier,
    #[error("Unable to deserialize element {0} at position {1}: {2}")]
    Deserialize(&'static str, u64, quick_xml::DeError),
    #[error("File reader reference is poisened")]
    PoisonedReader,
    #[error("File reader not initialized")]
    UninitializedReader,
    #[error("Unable to move reader to pos {0}, {1}")]
    Seek(u64, String),
}
