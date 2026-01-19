use std::io::BufRead;

use crate::{
    elements::{
        mz_ident_ml::MzIdentMl, sequence_collection::SequenceCollection,
        spectrum_identification_list::SpectrumIdentificationList,
    },
    error::Error,
};

/// Controlled vocabularies needed for mzIdentML validation.
pub mod controlled_vocabularies;
pub mod elements;
pub mod error;
/// Elements which only holds a reference to the position in the file to lower memory usage.
pub mod indexed_elements;
/// Module for preindexing a mzIdentML once to support random and streaming access
pub mod indexer;
/// Parsers for several attributes in the elements
pub mod parsing;
pub mod utils;

pub fn read<R: BufRead>(
    reader: R,
) -> Result<MzIdentMl<SequenceCollection, SpectrumIdentificationList>, Error> {
    let mut mzid_deserializer = quick_xml::de::Deserializer::from_reader(reader);
    serde_path_to_error::deserialize(&mut mzid_deserializer).map_err(Error::Deserialization)
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use super::read;

    pub static MZID_FILE_PATHS: &[&str] = &[
        "./test_data/scores_and_thresholds_1_3_0_draft.mzid",
        "./test_data/novor_v3.40.910_202512_results.mzid",
        "./test_data/byonic_v5.1.mzid",
    ];

    #[test]
    fn test_read() {
        for path in MZID_FILE_PATHS {
            let reader = BufReader::new(File::open(path).unwrap());

            let mzid_res = read(reader);
            assert!(mzid_res.is_ok(), "{path}: {}", mzid_res.unwrap_err());
        }
    }

    #[test]
    #[ignore = "Discrepancies between specificatio document and example."]
    fn test_validation() {
        for path in MZID_FILE_PATHS {
            let reader = BufReader::new(File::open(path).unwrap());

            let mzid = read(reader).unwrap();
            let validation_res = mzid.validate_document(true);
            assert!(
                validation_res.is_ok(),
                "{path}: {}",
                validation_res.unwrap_err()
            );
        }
    }
}
