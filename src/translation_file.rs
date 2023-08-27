use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
};

use serde_json::{Map, Value};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct TranslationFile {
    pub path: PathBuf,
    pub entries: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum TranslationFileError {
    #[error("key \"{0}\" is empty")]
    EmptyValue(String),
    #[error("key \"{key}\" is missing from {missing_in}")]
    MissingKey { key: String, missing_in: PathBuf },
    #[error("duplicate keys found in {0}, keys: {1:?}")]
    DuplicateKeys(PathBuf, Vec<String>),
}

impl TranslationFile {
    pub fn new(path: PathBuf) -> Result<Self, TranslationFileError> {
        let duplicates = find_key_duplicates(&path);
        if !duplicates.is_empty() {
            return Err(TranslationFileError::DuplicateKeys(path, duplicates));
        }

        let file = std::fs::File::open(&path).expect("Unable to open file");
        let reader = std::io::BufReader::new(file);
        let entries = serde_json::from_reader(reader).unwrap();
        Ok(Self { path, entries })
    }

    /// Compare two translation files and return an error if they are not compatible.
    ///
    /// Two translation files are compatible if:
    /// - They have the same keys
    /// - All keys have a non-empty value
    pub fn is_compatible_with(
        &self,
        other: &Self,
    ) -> Result<(), (Vec<TranslationFileError>, Vec<TranslationFileError>)> {
        let self_errors = self.check_rules(other);
        let other_errors = other.check_rules(self);

        if self_errors.is_empty() && other_errors.is_empty() {
            Ok(())
        } else {
            Err((self_errors, other_errors))
        }
    }

    fn check_rules(&self, other: &Self) -> Vec<TranslationFileError> {
        let mut errors = Vec::new();

        for (key, value) in &self.entries {
            // Check matching keys
            if !other.entries.contains_key(key) {
                errors.push(TranslationFileError::MissingKey {
                    key: key.clone(),
                    missing_in: other.path.clone(),
                });
            // Check non-empty values
            } else if value.is_empty() {
                errors.push(TranslationFileError::EmptyValue(key.to_string()));
            }
        }

        errors
    }

    pub fn sort_keys(&self) -> Result<(), Box<dyn std::error::Error>> {
        sort_json_keys(self.path.to_str().unwrap())
    }
}

fn find_key_duplicates(json_reader: &PathBuf) -> Vec<String> {
    let json_reader = File::open(json_reader).expect("Unable to open file");
    let lines = BufReader::new(json_reader).lines();

    let mut set = std::collections::HashSet::new();
    let mut duplicates = Vec::new();

    for line in lines.flatten().filter(|l| !l.trim().is_empty()) {
        let key = line
            .split(':')
            .next()
            .unwrap()
            .trim()
            .trim_matches('"')
            .to_string();

        if !set.insert(key.clone()) {
            duplicates.push(key);
        }
    }

    duplicates
}

fn sort_json_keys(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read the content from the given file
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Parse the content as a JSON object
    let json_value: Value = serde_json::from_str(&content)?;
    let object = json_value.as_object().unwrap();

    // Sort the keys using BTreeMap
    let sorted_map: BTreeMap<String, Value> =
        object.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

    // Convert back to Value
    let sorted_value = Value::Object(sorted_map.into_iter().collect::<Map<String, Value>>());

    // Convert back to JSON string
    let sorted_json = serde_json::to_string_pretty(&sorted_value)?;

    // Write the sorted JSON back to the file
    let mut file = File::create(file_path)?;
    file.write_all(sorted_json.as_bytes())?;

    Ok(())
}
