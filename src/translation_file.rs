use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct TranslationFile {
    pub path: PathBuf,
    pub entries: BTreeMap<String, String>,
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
        let entries = serde_json::from_reader(file).expect("Unable to parse json");

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

        if !self_errors.is_empty() || !other_errors.is_empty() {
            return Err((self_errors, other_errors));
        }

        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        let serialized_entries = serde_json::to_string_pretty(&self.entries)?;

        let mut file = File::create(&self.path)?;
        Ok(file.write_all(serialized_entries.as_bytes())?)
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
}

fn find_key_duplicates(json_path: &PathBuf) -> Vec<String> {
    let json_reader = File::open(json_path).expect("Unable to open file");
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
