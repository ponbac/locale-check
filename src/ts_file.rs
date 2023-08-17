use std::{fs::File, io::BufRead, path::Path};

pub struct TSFile {
    pub file: File,
}

impl TSFile {
    pub fn new(path: &Path) -> Self {
        let file = std::fs::File::open(path).expect("Unable to open file");

        Self { file }
    }

    /// Find all usages of `<FormattedMessage />` in a file, and return
    /// the id (e.g. `id="CO2_title"`) of the translation key used as
    /// well as the line number.
    pub fn find_formatted_message_usages(&self) -> Vec<(usize, String)> {
        let mut translation_key_usages: Vec<(usize, String)> = Vec::new();

        let mut looking_for_id = false;
        for (line_number, line) in std::io::BufReader::new(&self.file).lines().enumerate() {
            match (line, looking_for_id) {
                (Ok(line), false) => {
                    if line.contains("<FormattedMessage") {
                        looking_for_id = true;

                        if line.contains("id=") {
                            let key = extract_id_from_line(&line);
                            if let Some(key) = key {
                                translation_key_usages.push((line_number, key));
                            }
                            looking_for_id = false;
                        }
                    }
                }
                (Ok(line), true) => {
                    if line.contains("id=") {
                        let key = extract_id_from_line(&line);
                        if let Some(key) = key {
                            translation_key_usages.push((line_number, key));
                        }
                        looking_for_id = false;
                    }
                }
                (Err(e), _) => {
                    println!("Error reading line: {}", e);
                }
            }
        }

        translation_key_usages
    }
}

fn extract_id_from_line(line: &str) -> Option<String> {
    let mut id = None;
    if line.contains("id=") {
        let first_split = line.split("id=\"").nth(1);
        if let Some(first_split) = first_split {
            let second_split = first_split.split('"').next();
            if let Some(second_split) = second_split {
                id = Some(second_split.to_string());
            } else {
                println!("Unable to split line: {}", line);
            }
        }
    }

    id
}
