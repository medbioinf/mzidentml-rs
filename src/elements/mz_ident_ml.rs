use serde::{Deserialize, Serialize};

use crate::{
    elements::{
        analysis_collection::AnalysisCollection,
        analysis_protocol_collection::AnalysisProtocolCollection,
        analysis_software_list::AnalysisSoftwareList, audit_collection::AuditCollection,
        bibliographic_reference::BibliographicReference, data_collection::DataCollection,
        provider::Provider, sequence_collection::SequenceCollection,
    },
    error::ValidationError,
    has_cv_params,
};

use super::{cv_list::CvList, cv_param::CvParam, is_element::IsElement};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MzIdentMl {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    // This is a workaround to get xsi-attributes running, see:
    // https://github.com/tafia/quick-xml/issues/553#issuecomment-1432966843
    #[serde(rename = "@xmlns:xsi", alias = "@xsi", default)]
    pub xmlns_xsi: String,
    // This is a workaround to get xsi-attributes running, see:
    // https://github.com/tafia/quick-xml/issues/553#issuecomment-1432966843
    #[serde(rename = "@xsi:schemaLocation", alias = "@schemaLocation", default)]
    pub xsi_schema_location: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    // TODO: implmement sem ver like struckt
    #[serde(rename = "@version")]
    pub version: String,
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
    pub sequence_collection: Option<SequenceCollection>,
    #[serde(rename = "AnalysisCollection")]
    pub analysis_collection: AnalysisCollection,
    #[serde(rename = "AnalysisProtocolCollection")]
    pub analysis_protocol_collection: AnalysisProtocolCollection,
    #[serde(rename = "DataCollection")]
    pub data_collection: DataCollection,
    #[serde(rename = "BibliographicReference")]
    pub bibliographic_reference: Option<BibliographicReference>,
}

impl IsElement for MzIdentMl {
    fn validate(&self, strict: bool) -> Result<(), ValidationError> {
        for cv_list in &self.cv_list.cv {
            cv_list.validate(strict)?;
        }
        self.validate_cv_params(strict)?;
        if let Some(analysis_software_list) = &self.analysis_software_list {
            analysis_software_list.validate(strict)?;
        }
        if let Some(provider) = &self.provider {
            provider.validate(strict)?;
        }
        if let Some(audit_collection) = &self.audit_collection {
            audit_collection.validate(strict)?;
        }
        if let Some(sequence_collection) = &self.sequence_collection {
            sequence_collection.validate(strict)?;
        }

        self.analysis_collection.validate(strict)?;

        self.analysis_protocol_collection.validate(strict)?;

        self.data_collection.validate(strict)?;

        if let Some(bibliographic_reference) = &self.bibliographic_reference {
            bibliographic_reference.validate(strict)?;
        }

        Ok(())
    }
}

has_cv_params!(MzIdentMl, cv_params);
