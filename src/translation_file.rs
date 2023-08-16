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
}
