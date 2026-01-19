use std::borrow::Cow;

use crate::error::CvError;

/// Splits a CV term's accession / id into the CV name and id part.
/// Accession is expected in `<cv_name>:<id>`
///
/// # Arguments
/// * `accession` - Term's accession / id
///
pub fn split_accession<'a>(accession: &'a str) -> Result<(Cow<'a, str>, usize), CvError> {
    let accession_split = accession.split(':').collect::<Vec<&str>>();
    if accession_split.len() != 2 {
        return Err(CvError::InvalidIdFormat(accession.to_string()));
    }

    let onthology = Cow::Borrowed(accession_split[0]);
    let number = accession_split[1].parse::<usize>().map_err(|_| {
        CvError::InvalidSecondPart(onthology.clone().into(), accession_split[1].to_string())
    })?;

    Ok((onthology, number))
}
