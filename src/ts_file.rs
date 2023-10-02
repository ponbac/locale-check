use nom::{
    bytes::complete::{tag, take_until},
    IResult,
};
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
    path::{Path, PathBuf},
};

use crate::fenced;

#[derive(Debug)]
pub struct TSFile {
    pub file: File,
    pub path: PathBuf,
}

#[derive(Debug, PartialEq)]
pub struct KeyUsage {
    pub key: String,
    pub line: usize,
    pub file_path: PathBuf,
}

impl TSFile {
    pub fn new(path: &Path) -> Self {
        let file = File::open(path).expect("Unable to open file");
        Self {
            file,
            path: path.to_path_buf(),
        }
    }

    pub fn find_formatted_message_usages(&mut self) -> Vec<KeyUsage> {
        self.find_usages("<FormattedMessage", "id=")
    }

    pub fn find_format_message_usages(&mut self) -> Vec<KeyUsage> {
        self.find_usages("formatMessage(", "id:")
    }

    /// Random usage patterns that are used in the codebase.
    ///
    /// TODO: These should probably be read from a config file.
    pub fn find_misc_usages(&mut self) -> Vec<KeyUsage> {
        let identifiers = [
            "translationId:",
            "translationKey:",
            "transId:",
            "pageTitleId=",
            "titleId=",
        ];
        self.find_usages_multiple_tags(identifiers)
    }

    fn find_usages(&mut self, opening_tag: &str, id_tag: &str) -> Vec<KeyUsage> {
        let mut results = Vec::new();
        let mut found_opening = false;
        let mut found_ternary = false;
        for (line_number, line_result) in BufReader::new(&self.file).lines().enumerate() {
            if let Ok(line) = line_result {
                if line.contains(opening_tag) {
                    found_opening = true;
                }

                if found_opening {
                    if let Ok((_, key)) = extract_id(&line, id_tag) {
                        results.push(KeyUsage {
                            key,
                            line: line_number + 1,
                            file_path: self.path.to_path_buf(),
                        });
                        found_ternary = false;
                        found_opening = false;
                    } else if line.contains('?') {
                        if let Ok((_, key)) = extract_quoted_string(&line) {
                            results.push(KeyUsage {
                                key,
                                line: line_number + 1,
                                file_path: self.path.to_path_buf(),
                            });
                        }
                        found_ternary = true;
                    } else if found_ternary && line.contains(':') {
                        if let Ok((_, key)) = extract_quoted_string(&line) {
                            results.push(KeyUsage {
                                key,
                                line: line_number + 1,
                                file_path: self.path.to_path_buf(),
                            });
                        }
                        found_ternary = false;
                        found_opening = false;
                    } else if line.contains("/>") || line.contains("</") {
                        // TODO: think about edge cases where this might not be true!
                        found_ternary = false;
                        found_opening = false;
                    }
                }
            }
        }

        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        results
    }

    fn find_usages_multiple_tags(&mut self, tags: [&str; 5]) -> Vec<KeyUsage> {
        let mut results = Vec::new();
        for (line_number, line_result) in BufReader::new(&self.file).lines().enumerate() {
            if let Ok(line) = line_result {
                for &tag_str in &tags {
                    if line.contains(tag_str) {
                        if let Ok((_, key)) = extract_id(&line, tag_str) {
                            results.push(KeyUsage {
                                key,
                                line: line_number + 1,
                                file_path: self.path.to_path_buf(),
                            });
                        }
                    }
                }
            }
        }

        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        results
    }
}

fn extract_id<'a>(input: &'a str, id_tag: &'a str) -> IResult<&'a str, String> {
    let (input, _) = take_until(id_tag)(input)?;
    let (input, _) = tag(id_tag)(input)?;

    let (input, _) = take_until("\"")(input)?;
    let (input, id) = fenced("\"", "\"")(input)?;

    Ok((input, id.to_string()))
}

fn extract_quoted_string(input: &str) -> IResult<&str, String> {
    let (input, _) = take_until("\"")(input)?;
    let (input, id) = fenced("\"", "\"")(input)?;

    Ok((input, id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_format_message_usages() {
        let path = Path::new("test_files/component.tsx");
        let mut ts_file = TSFile::new(path);
        let actual = ts_file.find_format_message_usages();
        let expected = vec![KeyUsage {
            key: "name".to_string(),
            line: 20,
            file_path: path.to_path_buf(),
        }];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_find_formatted_message_usages() {
        let path = Path::new("test_files/component.tsx");
        let mut ts_file = TSFile::new(path);
        let actual = ts_file.find_formatted_message_usages();
        let expected = vec![
            KeyUsage {
                key: "name".to_string(),
                line: 22,
                file_path: path.to_path_buf(),
            },
            KeyUsage {
                key: "name".to_string(),
                line: 23,
                file_path: path.to_path_buf(),
            },
        ];
        assert_eq!(expected, actual);

        // let path = Path::new("test_files/select-component.tsx");
        // let mut ts_file = TSFile::new(path);
        // let actual = ts_file.find_formatted_message_usages();
        // for usage in &actual {
        //     println!("{:?}", usage.key);
        // }
    }

    #[test]
    fn test_extract_id_colon() {
        let input = r#"translationId: "some_id""#;
        let expected = "some_id".to_string();
        let (_, actual) = extract_id(input, "translationId: ").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_extract_id_equals() {
        let input = r#"translationId="some_id""#;
        let expected = "some_id".to_string();
        let (_, actual) = extract_id(input, "translationId=").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_extract_id_braces() {
        let input = r#"translationId={"some_id"}"#;
        let expected = "some_id".to_string();
        let (_, actual) = extract_id(input, "translationId=").unwrap();
        assert_eq!(expected, actual);
    }
}
