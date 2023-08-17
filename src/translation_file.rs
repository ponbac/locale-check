// {
//     "no_translation": "No translation",
//     "rental_title": "Rentals",
//     "economy_title": "Economy",
//     "sustainability_title": "Sustainability",
//     "customerarticles_title": "Own articles",
//     "moduletemplate_title": "Template for new modules",
//     "title": "Title"
// }

use std::{collections::HashMap, path::PathBuf};

pub struct TranslationFile {
    _path: PathBuf,
    pub entries: HashMap<String, String>,
}

impl TranslationFile {
    pub fn new(path: PathBuf) -> Self {
        let file = std::fs::File::open(&path).expect("Unable to open file");
        let reader = std::io::BufReader::new(file);

        let entries = serde_json::from_reader(reader).unwrap();
        Self {
            _path: path,
            entries,
        }
    }

    pub fn is_compatible_with(&self, other: &Self) -> Result<(), Vec<String>> {
        let mut errors = self.compare_with(other);
        errors.extend(other.compare_with(self));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn compare_with(&self, other: &Self) -> Vec<String> {
        let mut errors = Vec::new();

        for (key, value) in &self.entries {
            if !other.entries.contains_key(key) {
                errors.push(format!(
                    "Key \"{}\" is missing from {}",
                    key,
                    other._path.to_str().unwrap()
                ));
            } else if other.entries.get(key).unwrap().is_empty() {
                errors.push(format!(
                    "Key \"{}\" is empty in {}",
                    key,
                    other._path.to_str().unwrap()
                ));
            } else if value.is_empty() {
                errors.push(format!(
                    "Key \"{}\" is empty in {}",
                    key,
                    self._path.to_str().unwrap()
                ));
            }
        }

        errors
    }
}
