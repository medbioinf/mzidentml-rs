use std::collections::HashMap;
use std::io::{Cursor, prelude::*};

use quick_xml::events::Event;
use serde::de::DeserializeOwned;

use crate::elements::analysis_collection::AnalysisCollection;
use crate::elements::analysis_data::AnalysisData;
use crate::elements::analysis_protocol_collection::AnalysisProtocolCollection;
use crate::elements::analysis_software_list::AnalysisSoftwareList;
use crate::elements::attributes::semver::SemVer;
use crate::elements::audit_collection::AuditCollection;
use crate::elements::bibliographic_reference::BibliographicReference;
use crate::elements::cv_list::CvList;
use crate::elements::cv_param::CvParam;
use crate::elements::data_collection::DataCollection;
use crate::elements::inputs::Inputs;
use crate::elements::is_element::{IsElement, element_path_to_string};
use crate::elements::mz_ident_ml::MzIdentMl;
use crate::elements::protein_detection_list::ProteinDetectionList;
use crate::elements::provider::Provider;
use crate::error::{IndexingError, ValidationError};
use crate::indexed_elements::sequence_collection::SequenceCollection;
use crate::indexed_elements::spectrum_identification_list::SpectrumIdentificationList;

/// Default chunk size to read from file (1MB)
const DEFAULT_BUFFER_SIZE: usize = 1024 * 1000;

/// Start of ID attribute
const ID_START: &[u8] = b"id=\"";

/// Attribute end tag
const ATTRIBUTE_END: &[u8] = b"\"";

pub type IndexedMzIdentMl = MzIdentMl<SequenceCollection, SpectrumIdentificationList>;
pub type IndexedDataColletion = DataCollection<SpectrumIdentificationList>;
pub type IndexedAnalysisData = AnalysisData<SpectrumIdentificationList>;

/// Reads an mzIdentML only partially. DBSequence, Peptide, PeptideEvidence and SpectrumIdentificationResult are indexed and get deserialized on demand.
/// The parent elements of the indexed elements provide methods to access those elements.
/// Use [read] to create the IndexedMzIdentML
///
pub struct Indexer<'a, R: Seek + BufRead> {
    /// Event buffer
    buffer: Vec<u8>,
    /// XML event reader
    xml_reader: quick_xml::Reader<&'a mut R>,
    /// Element path
    element_path: Vec<String>,
}

impl<'a, R: Seek + BufRead> Indexer<'a, R> {
    /// Creates a new indexer
    ///
    /// # Arguments
    /// * `mzid_file` - Byte reader
    /// * `buffer_size` - Optional buffer size.
    ///
    fn new(mzid_file: &'a mut R, buffer_size: Option<usize>) -> Result<Self, IndexingError> {
        mzid_file
            .seek(std::io::SeekFrom::Start(0))
            .map_err(|err| IndexingError::MoveCursor(0, err))?;

        Ok(Self {
            buffer: Vec::with_capacity(buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE)),
            xml_reader: quick_xml::Reader::from_reader(mzid_file),
            element_path: Vec::with_capacity(10), // TODO: Lookup max depth of mzIdentML
        })
    }

    /// Read the given mzIdentML by indexing some elements to access on demand.
    /// This reduces memory usage significantly.
    ///
    /// # Arguments
    /// * `reader`- Open reader.
    /// * `buffer_size` - Size of the chunks to read from the file.
    ///
    pub fn read(
        mzid_file: &'a mut R,
        buffer_size: Option<usize>,
    ) -> Result<IndexedMzIdentMl, IndexingError> {
        Self::new(mzid_file, buffer_size)?.create_idx()
    }

    /// Reads the next XML event
    ///
    fn read_event(&mut self) -> Result<Event<'_>, IndexingError> {
        self.xml_reader
            .read_event_into(&mut self.buffer)
            .map_err(IndexingError::Xml)
    }

    /// Creates the index
    ///
    fn create_idx(&mut self) -> Result<IndexedMzIdentMl, IndexingError> {
        let mut xmlns: Option<String> = None;
        let mut xmlns_xsi: String = String::new();
        let mut xsi_schema_location: String = String::new();
        let mut id: Option<String> = None;
        let mut name: Option<String> = None;
        let mut version: Option<SemVer> = None;
        let mut cv_list: Option<CvList> = None;
        let mut cv_params: Vec<CvParam> = vec![];
        let mut analysis_software_list: Option<AnalysisSoftwareList> = None;
        let mut provider: Option<Provider> = None;
        let mut audit_collection: Option<AuditCollection> = None;
        let mut sequence_collection: Option<SequenceCollection> = None;
        let mut analysis_collection: Option<AnalysisCollection> = None;
        let mut analysis_protocol_collection: Option<AnalysisProtocolCollection> = None;
        let mut data_collection: Option<IndexedDataColletion> = None;
        let mut bibliographic_reference: Option<BibliographicReference> = None;

        self.element_path
            .push(IndexedMzIdentMl::ELEMENT_TAG.to_string());
        loop {
            let event = self.read_event()?.into_owned();
            match event {
                // Process the various elements. Some of them are just read to their end tag and get serialized in one. Some of them get special indexing function to index the some of the child elements.
                // Start event with ByteStart is an opening tag `<Tag attr1="1" ... attrN>`
                quick_xml::events::Event::Start(ref e) => match e.local_name().as_ref() {
                    b"MzIdentML" => {
                        // Deserialize the mzIdentML attributes
                        for attr in e.attributes() {
                            let attr = attr.map_err(IndexingError::Attribute)?;
                            match attr.key.local_name().as_ref() {
                                b"xmlns" => {
                                    xmlns = Some(
                                        String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                                    )
                                }
                                b"xsi" => {
                                    xmlns_xsi =
                                        String::from_utf8_lossy(attr.value.as_ref()).to_string()
                                }
                                b"id" => {
                                    id = Some(
                                        String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                                    )
                                }
                                b"schemaLocation" => {
                                    xsi_schema_location =
                                        String::from_utf8_lossy(attr.value.as_ref()).to_string()
                                }
                                b"version" => {
                                    version = Some(SemVer::try_from(
                                        String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                                    )?)
                                }
                                b"name" => {
                                    name = Some(
                                        String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                                    )
                                }
                                _ => {}
                            }
                        }
                    }
                    b"cvList" => {
                        cv_list = Some(self.deserialize_element(event)?);
                    }
                    b"AnalysisSoftwareList" => {
                        analysis_software_list = Some(self.deserialize_element(event)?);
                    }
                    b"Provider" => {
                        provider = Some(self.deserialize_element(event)?);
                    }
                    b"AuditCollection" => {
                        audit_collection = Some(self.deserialize_element(event)?);
                    }
                    b"SequenceCollection" => {
                        sequence_collection = Some(self.deserialize_sequence_collection(event)?);
                    }
                    b"AnalysisCollection" => {
                        analysis_collection = Some(self.deserialize_element(event)?);
                    }
                    b"AnalysisProtocolCollection" => {
                        analysis_protocol_collection = Some(self.deserialize_element(event)?);
                    }
                    b"DataCollection" => {
                        data_collection = Some(self.deserialize_data_collection(event)?);
                    }
                    b"BibliographicReference" => {
                        bibliographic_reference = Some(self.deserialize_element(event)?);
                    }
                    b"cvParam" => {
                        cv_params.push(self.deserialize_element(event)?);
                    }
                    _ => (),
                },
                // Empty event with ByteStart is a tag without children or text `<Tag attr1="1" ... attrN />`
                quick_xml::events::Event::Empty(ref e) => match e.local_name().as_ref() {
                    b"cvList" => {
                        cv_list = Some(self.deserialize_empty(event)?);
                    }
                    b"Provider" => {
                        provider = Some(self.deserialize_empty(event)?);
                    }
                    b"AuditCollection" => {
                        audit_collection = Some(self.deserialize_empty(event)?);
                    }
                    b"SequenceCollection" => {
                        sequence_collection = Some(self.deserialize_empty(event)?);
                    }
                    b"AnalysisCollection" => {
                        analysis_collection = Some(self.deserialize_empty(event)?);
                    }
                    b"AnalysisProtocolCollection" => {
                        analysis_protocol_collection = Some(self.deserialize_empty(event)?);
                    }
                    b"DataCollection" => {
                        data_collection = Some(self.deserialize_empty(event)?);
                    }
                    b"BibliographicReference" => {
                        bibliographic_reference = Some(self.deserialize_empty(event)?);
                    }
                    b"cvParam" => {
                        cv_params.push(self.deserialize_empty(event)?);
                    }
                    _ => (),
                },
                quick_xml::events::Event::Eof => break,
                _ => (),
            }
        }

        Ok(MzIdentMl {
            name,
            xmlns_xsi,
            xsi_schema_location,
            cv_params,
            analysis_software_list,
            provider,
            audit_collection,
            sequence_collection,
            xmlns: xmlns.ok_or(ValidationError::EmptyAttribute(
                self.element_path_to_string(),
                "xmlns",
            ))?,
            id: id.ok_or(ValidationError::EmptyAttribute(
                self.element_path_to_string(),
                "id",
            ))?,
            version: version.ok_or(ValidationError::EmptyAttribute(
                self.element_path_to_string(),
                "version",
            ))?,
            cv_list: cv_list.ok_or(ValidationError::MissingChild(
                self.element_path_to_string(),
                "cvList",
            ))?,
            analysis_collection: analysis_collection.ok_or(ValidationError::MissingChild(
                self.element_path_to_string(),
                "AnalysisCollection",
            ))?,
            analysis_protocol_collection: analysis_protocol_collection.ok_or(
                ValidationError::MissingChild(
                    self.element_path_to_string(),
                    "AnalysisProtocolCollection",
                ),
            )?,
            data_collection: data_collection.ok_or(ValidationError::MissingChild(
                self.element_path_to_string(),
                "DataCollection",
            ))?,
            bibliographic_reference,
        })
    }

    /// Deserializes the given empty element into T
    ///
    /// # Attributes
    /// `start` - Start event with StartBytes
    ///
    fn deserialize_empty<T: DeserializeOwned>(
        &mut self,
        start: Event<'_>,
    ) -> Result<T, IndexingError> {
        let tag = match start {
            Event::Empty(ref e) => e.local_name().as_ref().to_vec(),
            _ => {
                return Err(IndexingError::InvalidEvent(
                    self.element_path_to_string(),
                    event_to_str(&start),
                    "start tag",
                ));
            }
        };

        self.element_path
            .push(String::from_utf8_lossy(&tag).to_string());

        let mut element_bytes: Vec<u8> = Vec::with_capacity(start.len());
        let mut writer = quick_xml::Writer::new(&mut element_bytes);
        writer.write_event(start).unwrap();
        drop(writer);
        let mut element_bytes = Cursor::new(element_bytes);

        let deserialized_element = quick_xml::de::from_reader::<_, T>(&mut element_bytes)
            .map_err(|err| IndexingError::Deserialization(self.element_path_to_string(), err))?;

        self.element_path.pop();

        Ok(deserialized_element)
    }

    /// Reads the xml until the matching end tag to the given start tag occurs and deserializes into T.
    ///
    /// # Attributes
    /// `start` - Start event with StartBytes
    ///
    fn deserialize_element<T: DeserializeOwned>(
        &mut self,
        start: Event<'_>,
    ) -> Result<T, IndexingError> {
        let end_tag = match start {
            Event::Start(ref e) => e.local_name().as_ref().to_vec(),
            _ => {
                return Err(IndexingError::InvalidEvent(
                    self.element_path_to_string(),
                    event_to_str(&start),
                    "start tag",
                ));
            }
        };

        self.element_path
            .push(String::from_utf8_lossy(&end_tag).to_string());

        let mut element_bytes: Vec<u8> = Vec::with_capacity(start.len());
        let mut writer = quick_xml::Writer::new(&mut element_bytes);
        writer.write_event(start).unwrap();

        loop {
            let event = self.read_event()?.into_owned();
            match event {
                quick_xml::events::Event::End(ref e) => {
                    let is_end_tag_match = e.local_name().as_ref() == end_tag;
                    writer.write_event(event).unwrap();
                    if is_end_tag_match {
                        break;
                    }
                }
                quick_xml::events::Event::Eof => {
                    return Err(IndexingError::UnexpectedEOF(
                        self.element_path_to_string(),
                        String::from_utf8_lossy(&end_tag).to_string(),
                    ));
                }
                e => writer.write_event(e).unwrap(),
            }
        }
        drop(writer);
        let mut element_bytes = Cursor::new(element_bytes);
        let deserialized_element = quick_xml::de::from_reader::<_, T>(&mut element_bytes)
            .map_err(|err| IndexingError::Deserialization(self.element_path_to_string(), err))?;

        self.element_path.pop();
        Ok(deserialized_element)
    }

    /// Reads the given sequence collection to the end and serializes it with DBSequence, Peptide and PeptideEvicence only indexed.
    ///
    /// # Attributes
    /// `start` - Start event with StartBytes
    ///
    fn deserialize_sequence_collection(
        &mut self,
        start: Event<'_>,
    ) -> Result<SequenceCollection, IndexingError> {
        let end_tag =
            self.check_expected_start_event_tag(&start, SequenceCollection::ELEMENT_TAG)?;

        let mut element_bytes: Vec<u8> = Vec::with_capacity(start.len());
        let mut writer = quick_xml::Writer::new(&mut element_bytes);
        writer.write_event(start).unwrap();
        let mut index: HashMap<String, HashMap<String, u64>> = HashMap::new();

        // Marker to skip writing to element bytes until the end tag is read.
        let mut in_sub_element: Vec<u8> = Vec::new();
        loop {
            let event = self.read_event()?.into_owned();
            match event {
                Event::Start(ref e) | Event::Empty(ref e) => match e.local_name().as_ref() {
                    b"Peptide" | b"PeptideEvidence" | b"DBSequence" => {
                        if let Event::Start(_) = &event {
                            in_sub_element = e.local_name().as_ref().to_vec()
                        }

                        let tag = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                        let id = get_id_attributes(e, self.xml_reader.buffer_position())?;
                        let position = Self::tag_position_for_index(
                            self.xml_reader.buffer_position(),
                            e.len() as u64,
                        );

                        index
                            .entry(tag)
                            .and_modify(|offsets| {
                                offsets.insert(id.clone(), position);
                            })
                            .or_insert(HashMap::from([(id, position)]));
                    }
                    _ => {
                        if !in_sub_element.is_empty() {
                            continue;
                        }
                        writer.write_event(event).unwrap();
                    }
                },
                Event::End(ref e) => {
                    if in_sub_element.is_empty() {
                        let is_end_tag_match = e.local_name().as_ref() == end_tag;
                        writer.write_event(event).unwrap();
                        if is_end_tag_match {
                            break;
                        }
                    } else if !in_sub_element.is_empty()
                        && in_sub_element != e.local_name().as_ref()
                    {
                        continue;
                    } else {
                        in_sub_element.clear();
                    }
                }
                Event::Eof => {
                    return Err(IndexingError::UnexpectedEOF(
                        self.element_path_to_string(),
                        String::from_utf8_lossy(&end_tag).to_string(),
                    ));
                }
                e => {
                    if !in_sub_element.is_empty() {
                        continue;
                    }
                    writer.write_event(e).unwrap();
                }
            }
        }
        drop(writer);
        let mut element_bytes = Cursor::new(element_bytes);
        let mut sequence_collection = quick_xml::de::from_reader::<_, SequenceCollection>(
            &mut element_bytes,
        )
        .map_err(|err| IndexingError::Deserialization(self.element_path_to_string(), err))?;

        if let Some(db_sequence_map) = index.remove("DBSequence") {
            sequence_collection.db_sequences_map = db_sequence_map;
        }
        if let Some(peptides_map) = index.remove("Peptide") {
            sequence_collection.peptides_map = peptides_map;
        }
        if let Some(peptide_evidence_map) = index.remove("PeptideEvidence") {
            sequence_collection.peptide_evidence_map = peptide_evidence_map;
        }
        Ok(sequence_collection)
    }

    fn deserialize_data_collection(
        &mut self,
        start: Event<'_>,
    ) -> Result<IndexedDataColletion, IndexingError> {
        let end_tag =
            self.check_expected_start_event_tag(&start, IndexedDataColletion::ELEMENT_TAG)?;

        let mut inputs: Option<Inputs> = None;
        let mut analysis_data: Option<IndexedAnalysisData> = None;

        loop {
            let event = self.read_event()?.into_owned();
            match event {
                Event::Start(ref e) => match e.local_name().as_ref() {
                    b"Inputs" => inputs = Some(self.deserialize_element(event)?),
                    b"AnalysisData" => analysis_data = Some(self.deserialize_analysis_data(event)?),
                    _ => (),
                },
                Event::Empty(ref e) => match e.local_name().as_ref() {
                    b"Inputs" => inputs = Some(self.deserialize_empty(event)?),
                    b"AnalysisData" => analysis_data = Some(self.deserialize_empty(event)?),
                    _ => (),
                },
                Event::End(ref e) => {
                    let is_end_tag_match = e.local_name().as_ref() == end_tag;
                    if is_end_tag_match {
                        break;
                    }
                }
                Event::Eof => {
                    return Err(IndexingError::UnexpectedEOF(
                        self.element_path_to_string(),
                        String::from_utf8_lossy(&end_tag).to_string(),
                    ));
                }
                _ => (),
            }
        }

        Ok(DataCollection {
            inputs: inputs.ok_or_else(|| {
                ValidationError::MissingChild(self.element_path_to_string(), "Inputs")
            })?,
            analysis_data: analysis_data.ok_or_else(|| {
                ValidationError::MissingChild(self.element_path_to_string(), "AnalysisData")
            })?,
        })
    }

    fn deserialize_analysis_data(
        &mut self,
        start: Event<'_>,
    ) -> Result<IndexedAnalysisData, IndexingError> {
        let end_tag =
            self.check_expected_start_event_tag(&start, IndexedAnalysisData::ELEMENT_TAG)?;

        let mut protein_detection_list: Option<ProteinDetectionList> = None;
        let mut spectrum_identification_lists: Vec<SpectrumIdentificationList> = Vec::new();

        loop {
            let event = self.read_event()?.into_owned();
            match event {
                Event::Start(ref e) => match e.local_name().as_ref() {
                    b"ProteinDetectionList" => {
                        protein_detection_list = Some(self.deserialize_element(event)?)
                    }
                    b"SpectrumIdentificationList" => spectrum_identification_lists
                        .push(self.deserialize_spectrum_identification_list(event)?),
                    _ => (),
                },
                Event::Empty(ref e) => match e.local_name().as_ref() {
                    b"ProteinDetectionList" => {
                        protein_detection_list = Some(self.deserialize_empty(event)?)
                    }
                    b"SpectrumIdentificationList" => {
                        spectrum_identification_lists.push(self.deserialize_empty(event)?)
                    }
                    _ => (),
                },
                Event::End(ref e) => {
                    let is_end_tag_match = e.local_name().as_ref() == end_tag;
                    if is_end_tag_match {
                        break;
                    }
                }
                Event::Eof => {
                    return Err(IndexingError::UnexpectedEOF(
                        self.element_path_to_string(),
                        String::from_utf8_lossy(&end_tag).to_string(),
                    ));
                }
                _ => (),
            }
        }

        Ok(IndexedAnalysisData {
            protein_detection_list,
            spectrum_identification_lists,
        })
    }

    fn deserialize_spectrum_identification_list(
        &mut self,
        start: Event<'_>,
    ) -> Result<SpectrumIdentificationList, IndexingError> {
        let end_tag =
            self.check_expected_start_event_tag(&start, SpectrumIdentificationList::ELEMENT_TAG)?;

        let mut element_bytes: Vec<u8> = Vec::with_capacity(start.len());
        let mut writer = quick_xml::Writer::new(&mut element_bytes);
        writer.write_event(start).unwrap();
        let mut index: HashMap<String, u64> = HashMap::new();

        let mut in_sub_element: Vec<u8> = Vec::new();
        loop {
            let event = self.read_event()?.into_owned();
            match event {
                Event::Start(ref e) | Event::Empty(ref e) => match e.local_name().as_ref() {
                    b"SpectrumIdentificationResult" => {
                        if let Event::Start(_) = &event {
                            in_sub_element = e.local_name().as_ref().to_vec()
                        }

                        let id = get_id_attributes(e, self.xml_reader.buffer_position())?;
                        let position = Self::tag_position_for_index(
                            self.xml_reader.buffer_position(),
                            e.len() as u64,
                        );

                        index.insert(id, position);
                    }
                    _ => {
                        if !in_sub_element.is_empty() {
                            continue;
                        }
                        writer.write_event(event).unwrap();
                    }
                },
                Event::End(ref e) => {
                    if in_sub_element.is_empty() {
                        let is_end_tag_match = e.local_name().as_ref() == end_tag;
                        writer.write_event(event).unwrap();
                        if is_end_tag_match {
                            break;
                        }
                    } else if !in_sub_element.is_empty()
                        && in_sub_element != e.local_name().as_ref()
                    {
                        continue;
                    } else {
                        in_sub_element.clear();
                    }
                }
                Event::Eof => {
                    return Err(IndexingError::UnexpectedEOF(
                        self.element_path_to_string(),
                        String::from_utf8_lossy(&end_tag).to_string(),
                    ));
                }
                e => {
                    if !in_sub_element.is_empty() {
                        continue;
                    }
                    writer.write_event(e).unwrap();
                }
            }
        }
        drop(writer);
        let mut element_bytes = Cursor::new(element_bytes);
        let mut spectrum_identification_list = quick_xml::de::from_reader::<
            _,
            SpectrumIdentificationList,
        >(&mut element_bytes)
        .map_err(|err| IndexingError::Deserialization(self.element_path_to_string(), err))?;

        spectrum_identification_list.spectrum_identification_results_map = index;

        Ok(spectrum_identification_list)
    }

    fn tag_position_for_index(reader_position: u64, tag_lenth: u64) -> u64 {
        reader_position - tag_lenth - 3
    }

    fn element_path_to_string(&self) -> String {
        element_path_to_string(&self.element_path)
    }

    fn check_expected_start_event_tag(
        &mut self,
        event: &Event<'_>,
        expected_tag: &'static str,
    ) -> Result<Vec<u8>, IndexingError> {
        let tag = match event {
            Event::Start(e) => e.local_name().as_ref().to_vec(),
            _ => {
                return Err(IndexingError::InvalidEvent(
                    self.element_path_to_string(),
                    event_to_str(event),
                    "start tag",
                ));
            }
        };
        let tag_str = String::from_utf8_lossy(&tag);
        if tag_str != expected_tag {
            return Err(IndexingError::InvalidTag(
                self.element_path_to_string(),
                tag_str.to_string(),
                expected_tag,
            ));
        }
        drop(tag_str);

        Ok(tag)
    }
}

pub fn get_id_attributes(id: &[u8], reader_position: u64) -> Result<String, IndexingError> {
    let id_start = id
        .windows(ID_START.len())
        .position(|x| x == ID_START)
        .ok_or(IndexingError::NoIdAttribute(reader_position))?
        + ID_START.len();
    let id_end = id[id_start..]
        .windows(ATTRIBUTE_END.len())
        .position(|x| x == ATTRIBUTE_END)
        .ok_or(IndexingError::IdAttributeNotClosed(reader_position))?
        + id_start;
    Ok(String::from_utf8_lossy(&id[id_start..id_end]).to_string())
}

pub fn event_to_str(event: &Event<'_>) -> &'static str {
    match event {
        Event::CData(_) => "CDATA",
        Event::Comment(_) => "comment",
        Event::Decl(_) => "XML declaration",
        Event::DocType(_) => "documentation type",
        Event::Empty(_) => "empty tag",
        Event::End(_) => "end tag",
        Event::GeneralRef(_) => "reference",
        Event::PI(_) => "processing instructions",
        Event::Start(_) => "start tag",
        Event::Text(_) => "text between tags",
        Event::Eof => "end of file",
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use crate::{indexer::Indexer, tests::MZID_FILE_PATHS};

    #[test]
    fn test_read() {
        for path in MZID_FILE_PATHS {
            let mut reader = BufReader::new(File::open(path).unwrap());

            let mzid_res = Indexer::read(&mut reader, None);
            assert!(mzid_res.is_ok(), "{path}: {}", mzid_res.unwrap_err());
        }
    }
}
