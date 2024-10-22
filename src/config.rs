use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub i18n_path: String,
    pub dictionary_file: DictionaryFile,
    pub usage_pattern: UsagePattern,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DictionaryFile {
    pub file_extension: String,
    pub variable_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UsagePattern {
    pub method_name: String,
    pub arguments: Vec<String>,
}
