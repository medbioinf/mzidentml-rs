# mzIdentML

Reader for [mzIdentML-files](https://www.psidev.info/mzidentml).

## Usage
The crate is for those who are familiar with the format and want to extract information to further processing. Files can be loaded directly into memory or indexed first to support random or streaming access to the data.  
The parser is relative relaxed and only complains about missing required attributes or missing children during file loading. In case 1 or many children of the same type should be present but missing, the parser will just create an empty vector. This is checked during optional validation.

### Validation
Validating is optional and has initial support for different version of the file format.

## Development
Scattered around the code are `TODO` comments which can be worked on. In general more detailed tests are needed, especially around the indexer.

Please submit your work as PRs.

### Errors and validations
*  Due to some discrepancies between the format specifications and the provided examples, the validation is not fully functional and can not be used for proper testing.
* The validation is not following references yet. To do so, the validation needs to have access to the (indexed) document.

### Format version
The parser already parses file in version 1.1, 1.2 and 1.3. However, some details in the validation might not be correct.

### Elements
The structs for the elements are completely hand crafted. In general, it might be a good idea to assemble feature complete mzIdentMl files for each version of the format. Then use [xml_schema_generator](https://github.com/Thomblin/xml_schema_generator) to generate the struct for each element automatically, clean them up and just implement the necessary traits.

### Test data
* [HUPO-PSI](https://www.psidev.info/mzidentml): scores_and_thresholds_1_3_0_draft.mzid
* [Douwe Schulte](https://github.com/douweschulte): `novor_v3.40.910_202512_results.mzid` & `novor_v3.40.910_202512_results.mzid`
