use std::io::{BufRead, Seek};

use serde::{Deserialize, Serialize};

use crate::{
    elements::{
        analysis_collection::AnalysisCollection,
        analysis_data::AnalysisData,
        analysis_protocol_collection::AnalysisProtocolCollection,
        analysis_software_list::AnalysisSoftwareList,
        attributes::semver::SemVer,
        audit_collection::AuditCollection,
        bibliographic_reference::BibliographicReference,
        data_collection::DataCollection,
        has_cv_params::{CvParamRule, HasCvParams},
        provider::Provider,
        sequence_collection::{IsSequenceCollection, SequenceCollection},
        spectrum_identification_list::{IsSpectrumIdentificationList, SpectrumIdentificationList},
    },
    error::ValidationError,
    indexed_elements::{
        is_indexed_element::IsIndexedElement,
        sequence_collection::SequenceCollection as IndexedSequenceCollection,
        spectrum_identification_list::SpectrumIdentificationList as IndexedSpectrumIdentificationList,
    },
};

use super::{cv_list::CvList, cv_param::CvParam, is_element::IsElement};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "SC: IsSequenceCollection, SIL: IsSpectrumIdentificationList")]
pub struct MzIdentMl<SC: IsSequenceCollection, SIL: IsSpectrumIdentificationList> {
    #[serde(default, rename = "@xmlns")]
    pub xmlns: String,
    // This is a workaround to get xsi-attributes running, see:
    // https://github.com/tafia/quick-xml/issues/553#issuecomment-1432966843
    #[serde(default, rename = "@xmlns:xsi", alias = "@xsi")]
    pub xmlns_xsi: String, // TODO: Probably unnecessary
    // This is a workaround to get xsi-attributes running, see:
    // https://github.com/tafia/quick-xml/issues/553#issuecomment-1432966843
    #[serde(default, rename = "@xsi:schemaLocation", alias = "@schemaLocation")]
    pub xsi_schema_location: String,
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@name")]
    pub name: Option<String>,
    // TODO: implement semver like struct
    #[serde(rename = "@version")]
    pub version: SemVer,
    #[serde(rename = "cvList")]
    pub cv_list: CvList,
    #[serde(default, rename = "cvParam")]
    pub cv_params: Vec<CvParam>,
    #[serde(rename = "AnalysisSoftwareList")]
    pub analysis_software_list: Option<AnalysisSoftwareList>,
    #[serde(rename = "Provider")]
    pub provider: Option<Provider>,
    #[serde(rename = "AuditCollection")]
    pub audit_collection: Option<AuditCollection>,
    #[serde(rename = "SequenceCollection")]
    pub sequence_collection: Option<SC>,
    #[serde(rename = "AnalysisCollection")]
    pub analysis_collection: AnalysisCollection,
    #[serde(rename = "AnalysisProtocolCollection")]
    pub analysis_protocol_collection: AnalysisProtocolCollection,
    #[serde(rename = "DataCollection")]
    pub data_collection: DataCollection<SIL>,
    #[serde(rename = "BibliographicReference")]
    pub bibliographic_reference: Option<BibliographicReference>,
}

impl MzIdentMl<SequenceCollection, SpectrumIdentificationList> {
    pub fn validate_document(&self, strict: bool) -> Result<(), ValidationError> {
        let mut elements_path: Vec<String> = Vec::with_capacity(10); // TODO: Lookup actual mzIdentML depth
        self.validate(&self.version, strict, &mut elements_path, None)
    }
}

impl MzIdentMl<IndexedSequenceCollection, IndexedSpectrumIdentificationList> {
    pub fn validate_document<R: BufRead + Seek>(
        &self,
        strict: bool,
        reader: &mut R,
    ) -> Result<(), ValidationError> {
        let mut elements_path: Vec<String> = Vec::with_capacity(10); // TODO: Lookup actual mzIdentML depth
        self.validate(&self.version, strict, &mut elements_path, None)?;
        elements_path.push(Self::ELEMENT_TAG.to_string());
        if let Some(sequence_collection) = &self.sequence_collection {
            sequence_collection.validate_indexed(
                &self.version,
                strict,
                &mut elements_path,
                None,
                reader,
            )?;
        }
        elements_path
            .push(DataCollection::<IndexedSpectrumIdentificationList>::ELEMENT_TAG.to_string());
        elements_path
            .push(AnalysisData::<IndexedSpectrumIdentificationList>::ELEMENT_TAG.to_string());
        for (elem_idx, elem) in self
            .data_collection
            .analysis_data
            .spectrum_identification_lists
            .iter()
            .enumerate()
        {
            elem.validate_indexed(
                &self.version,
                strict,
                &mut elements_path,
                Some(elem_idx),
                reader,
            )?;
        }

        Ok(())
    }
}

impl<SC: IsSequenceCollection, SIL: IsSpectrumIdentificationList> IsElement for MzIdentMl<SC, SIL> {
    const ELEMENT_TAG: &str = "MzIdentMl";

    fn inner_validate(
        &self,
        version: &SemVer,
        strict: bool,
        element_path: &mut Vec<String>,
    ) -> Result<(), ValidationError> {
        self.cv_list.validate(version, strict, element_path, None)?;

        self.validate_cv_params(version, strict, element_path)?;
        if let Some(analysis_software_list) = &self.analysis_software_list {
            analysis_software_list.validate(version, strict, element_path, None)?;
        }
        if let Some(provider) = &self.provider {
            provider.validate(version, strict, element_path, None)?;
        }
        if let Some(audit_collection) = &self.audit_collection {
            audit_collection.validate(version, strict, element_path, None)?;
        }
        if let Some(sequence_collection) = &self.sequence_collection {
            sequence_collection.validate(version, strict, element_path, None)?;
        }

        self.analysis_collection
            .validate(version, strict, element_path, None)?;

        self.analysis_protocol_collection
            .validate(version, strict, element_path, None)?;

        self.data_collection
            .validate(version, strict, element_path, None)?;

        if let Some(bibliographic_reference) = &self.bibliographic_reference {
            bibliographic_reference.validate(version, strict, element_path, None)?;
        }

        Ok(())
    }
}

impl<SC: IsSequenceCollection, SIL: IsSpectrumIdentificationList> HasCvParams
    for MzIdentMl<SC, SIL>
{
    const CV_PARAM_RULES: &[CvParamRule] = &[];

    fn cv_params(&self) -> impl Iterator<Item = &CvParam> {
        self.cv_params.iter()
    }
}
