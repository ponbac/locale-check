use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use thiserror::Error;

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
}

impl TranslationFile {
    pub fn new(path: PathBuf) -> Self {
        let duplicates = find_key_duplicates(&path);
        if !duplicates.is_empty() {
            println!("duplicate keys found in file {:?}: {:?}", path, duplicates);
            std::process::exit(1);
        }

        let file = std::fs::File::open(&path).expect("Unable to open file");
        let reader = std::io::BufReader::new(file);
        let entries = serde_json::from_reader(reader).unwrap();
        Self { path, entries }
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
}

fn find_key_duplicates(json_reader: &PathBuf) -> Vec<String> {
    let json_reader = File::open(json_reader).expect("Unable to open file");
    let lines = BufReader::new(json_reader).lines();

    let mut set = std::collections::HashSet::new();
    let mut duplicates = Vec::new();

    for line in lines.flatten() {
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
