use nom::{
    bytes::complete::{tag, take_until},
    IResult,
};
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
    path::Path,
};

use crate::fenced;

pub struct TSFile {
    pub file: File,
}

impl TSFile {
    pub fn new(path: &Path) -> Self {
        let file = File::open(path).expect("Unable to open file");
        Self { file }
    }

    pub fn find_formatted_message_usages(&mut self) -> Vec<(usize, String)> {
        self.find_usages("<FormattedMessage", "id=")
    }

    pub fn find_format_message_usages(&mut self) -> Vec<(usize, String)> {
        self.find_usages("formatMessage(", "id:")
    }

    /// Random usage patterns that are not covered by the other methods.
    ///
    /// TODO: Should read these from a config file probably!
    pub fn find_misc_usages(&mut self) -> Vec<(usize, String)> {
        let identifiers = [
            "translationId:",
            "translationKey:",
            "transId:",
            "pageTitleId=",
            "titleId=",
        ];
        self.find_usages_multiple_tags(identifiers)
    }

    fn find_usages(&mut self, opening_tag: &str, id_tag: &str) -> Vec<(usize, String)> {
        let mut results = Vec::new();
        let mut found_opening = false;
        for (line_number, line_result) in BufReader::new(&self.file).lines().enumerate() {
            if let Ok(line) = line_result {
                if line.contains(opening_tag) {
                    found_opening = true;
                }

                if found_opening {
                    if let Ok((_, key)) = extract_id(&line, id_tag) {
                        results.push((line_number + 1, key));
                        found_opening = false;
                    }
                }
            }
        }

        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        results
    }

    fn find_usages_multiple_tags(&mut self, tags: [&str; 5]) -> Vec<(usize, String)> {
        let mut results = Vec::new();
        for (line_number, line_result) in BufReader::new(&self.file).lines().enumerate() {
            if let Ok(line) = line_result {
                for &tag_str in &tags {
                    if line.contains(tag_str) {
                        if let Ok((_, key)) = extract_id(&line, tag_str) {
                            results.push((line_number + 1, key));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_format_message_usages() {
        let mut ts_file = TSFile::new(Path::new("test_files/component.tsx"));
        let actual = ts_file.find_format_message_usages();
        let expected = vec![(20, "name".to_string())];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_find_formatted_message_usages() {
        let mut ts_file = TSFile::new(Path::new("test_files/component.tsx"));
        let actual = ts_file.find_formatted_message_usages();
        let expected = vec![(22, "name".to_string()), (23, "name".to_string())];
        assert_eq!(expected, actual);
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
