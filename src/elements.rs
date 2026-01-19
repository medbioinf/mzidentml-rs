pub mod additional_search_params;
pub mod affiliation;
pub mod ambiguous_residue;
pub mod analysis_collection;
pub mod analysis_data;
pub mod analysis_params;
pub mod analysis_protocol_collection;
pub mod analysis_software;
pub mod analysis_software_list;
/// Element attributes which contains more than basic types
pub mod attributes;
pub mod audit_collection;
pub mod bibliographic_reference;
pub mod contact_role;
pub mod customizations;
pub mod cv;
pub mod cv_list;
pub mod cv_param;
pub mod data_collection;
pub mod database_filters;
pub mod database_name;
pub mod database_translation;
pub mod db_sequence;
pub mod enzyme;
pub mod enzyme_name;
pub mod enzymes;
pub mod exclude;
pub mod external_format_documentation;
pub mod file_format;
pub mod filter;
pub mod filter_type;
pub mod fragment_array;
pub mod fragment_tolerance;
pub mod fragmentation;
pub mod fragmentation_table;
pub mod include;
pub mod input_spectra;
pub mod input_spectrum_identifications;
pub mod inputs;
pub mod ions_type;
pub mod mass_table;
pub mod measure;
pub mod modification;
pub mod modification_params;
pub mod mz_ident_ml;
pub mod organization;
pub mod parent;
pub mod parent_tolerance;
pub mod peptide;
pub mod peptide_evidence;
pub mod peptide_evidence_ref;
pub mod peptide_hypothesis;
pub mod peptide_sequence;
pub mod person;
pub mod protein_ambiguity_group;
pub mod protein_detection;
pub mod protein_detection_hypothesis;
pub mod protein_detection_list;
pub mod protein_detection_protocol;
pub mod provider;
pub mod residue;
pub mod role;
pub mod search_database;
pub mod search_database_ref;
pub mod search_modification;
pub mod search_type;
pub mod seq;
pub mod sequence_collection;
pub mod site_regexp;
pub mod software_name;
pub mod source_file;
pub mod specificity_rules;
pub mod spectra_data;
pub mod spectrum_id_format;
pub mod spectrum_identification;
pub mod spectrum_identification_item_ref;
pub mod spectrum_identification_list;
pub mod spectrum_identification_protocol;
pub mod spectrum_identification_result;
pub mod substitution_modification;
pub mod threshold;
pub mod translation_table;
pub mod user_param;

pub mod has_cv_params;
pub mod is_element;
pub mod is_list;
mod spectrum_identification_item;

use std::{collections::HashSet, sync::LazyLock};

pub static ALLOWED_SUBSTITUTION_RESIDUES: LazyLock<HashSet<char>> = LazyLock::new(|| {
    let residues = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '?', '-',
    ];
    residues.iter().cloned().collect()
});

// Elements to implement
// <MzIdentML>
// <AdditionalSearchParams>
// <Affiliation>
// <AmbiguousResidue>
// <AnalysisCollection>
// <AnalysisData>
// <AnalysisParams>
// <AnalysisProtocolCollection>
// <AnalysisSampleCollection>
// <AnalysisSoftware>
// <AnalysisSoftwareList>
// <AuditCollection>
// <BibliographicReference>
// <ContactRole>
// <Customizations>
// <cv>
// <cvList>
// <cvParam>
// <DatabaseFilters>
// <DatabaseName>
// <DatabaseTranslation>
// <DataCollection>
// <DBSequence>
// <Enzyme>
// <EnzymeName>
// <Enzymes>
// <Exclude>
// <ExternalFormatDocumentation>
// <FileFormat>
// <Filter>
// <FilterType>
// <FragmentArray>
// <Fragmentation>
// <FragmentationTable>
// <FragmentTolerance>
// <Include>
// <Inputs>
// <InputSpectra>
// <InputSpectrumIdentifications>
// <IonType>
// <MassTable>
// <Measure>
// <Modification>
// <ModificationParams>
// <Organization>
// <Parent>
// <ParentTolerance>
// <Peptide>
// <PeptideEvidence>
// <PeptideEvidenceRef>
// <PeptideHypothesis>
// <PeptideSequence>
// <Person>
// <ProteinAmbiguityGroup>
// <ProteinDetection>
// <ProteinDetectionHypothesis>
// <ProteinDetectionList>
// <ProteinDetectionProtocol>
// <Provider>
// <Residue>
// <Role>
// <Sample>
// <SearchDatabase>
// <SearchDatabaseRef>
// <SearchModification>
// <SearchType>
// <Seq>
// <SequenceCollection>
// <SiteRegexp>
// <SoftwareName>
// <SourceFile>
// <SpecificityRules>
// <SpectraData>
// <SpectrumIdentification>
// <SpectrumIdentificationItem>
// <SpectrumIdentificationItemRef>
// <SpectrumIdentificationList>
// <SpectrumIdentificationProtocol>
// <SpectrumIdentificationResult>
// <SpectrumIDFormat>
// <SubSample>
// <SubstitutionModification>
// <Threshold>
// <TranslationTable>
// <userParam>
