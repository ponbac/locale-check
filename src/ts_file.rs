use nom::{
    bytes::complete::{tag, take_until},
    character::complete::multispace0,
    combinator::opt,
    error::Error,
    IResult,
};
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
    path::Path,
};

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
        self.find_usages("formatMessage({", "id:")
    }

    pub fn find_misc_usages(&mut self) -> Vec<(usize, String)> {
        let identifiers = ["translationId:", "translationKey:", "transId:"];
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

    fn find_usages_multiple_tags(&mut self, tags: [&str; 3]) -> Vec<(usize, String)> {
        let mut results = Vec::new();
        for (line_number, line_result) in BufReader::new(&self.file).lines().enumerate() {
            if let Ok(line) = line_result {
                for &tag_str in &tags {
                    if tag::<_, _, Error<&str>>(tag_str)(line.as_str()).is_ok() {
                        if let Ok((_, key)) = extract_id(line.as_str(), tag_str) {
                            results.push((line_number, key));
                            break;
                        }
                    }
                }
            }
        }

        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        results
    }
}

/// Discard everything before the end of the opening tag
// fn contains_opening_tag<'a>(input: &'a str, opening_tag: &'a str) -> IResult<&'a str, &'a str> {
//     let (input, _) = opt(multispace0)(input)?;
//     let (input, _) = take_until(opening_tag)(input)?;
//     let (remaining_input, _) = tag(opening_tag)(input)?;
//     Ok((remaining_input, opening_tag))
// }

/// Extract the id from the line
/// e.g. {intl.formatMessage({ id: "name" })}...
/// returns "name"
fn extract_id<'a>(input: &'a str, id_tag: &'a str) -> IResult<&'a str, String> {
    let (input, _) = opt(multispace0)(input)?;
    let (input, _) = take_until(id_tag)(input)?;
    let (input, _) = tag(id_tag)(input)?;
    let (input, _) = opt(multispace0)(input)?;
    let (input, _) = tag("\"")(input)?;
    let (input, id) = take_until("\"")(input)?;
    let (input, _) = tag("\"")(input)?;
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
    fn test_extract_id() {
        let input = r#"translationId: "some_id""#;
        let expected = "some_id".to_string();
        let (_, actual) = extract_id(input, "translationId: ").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_extract_id_with_whitespace_and_newline() {
        let input = r#"
        translationId: "some_id"
        "#;
        let expected = "some_id".to_string();
        let (_, actual) = extract_id(input, "translationId: ").unwrap();
        assert_eq!(expected, actual);
    }
}
