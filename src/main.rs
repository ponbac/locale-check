use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use clap::Parser;
use locale_check::translation_file::TranslationFile;
use walkdir::{DirEntry, WalkDir};

/// Handle those damn translations...
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Root directory to search from
    #[arg(short, long, default_value = ".")]
    root_dir: PathBuf,
    /// Path to English translation file
    #[arg(short, long)]
    en_file: PathBuf,
    /// Path to Swedish translation file
    #[arg(short, long)]
    sv_file: PathBuf,
}

static EXTENSIONS_TO_SEARCH: [&str; 2] = ["ts", "tsx"];

fn main() {
    let args = Args::parse();

    let en_translation_file = TranslationFile::new(args.en_file);
    let sv_translation_file = TranslationFile::new(args.sv_file);
    if let Err(errors) = en_translation_file.is_compatible_with(&sv_translation_file) {
        for error in errors {
            println!("{}", error);
        }
        std::process::exit(1);
    }
    if let Err(errors) = sv_translation_file.is_compatible_with(&en_translation_file) {
        for error in errors {
            println!("{}", error);
        }
        std::process::exit(1);
    }

    let walker = WalkDir::new(args.root_dir)
        .into_iter()
        // Exclude node_modules
        .filter_entry(|e| !is_node_modules(e))
        // Filter out any non-accessible files
        .filter_map(|e| e.ok());

    for entry in walker {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if EXTENSIONS_TO_SEARCH.contains(&ext.to_str().unwrap()) {
                    let file = File::open(path).expect("Unable to open file");
                    let reader = std::io::BufReader::new(file);

                    let key_usages = find_formatted_message_usages(reader);
                    for (line_number, id) in key_usages {
                        println!(
                            "[{}] \"{}\" on line {}",
                            path.file_name().unwrap().to_str().unwrap(),
                            id,
                            line_number
                        );
                        println!(
                            "  EN: {}",
                            en_translation_file
                                .entries
                                .get(&id)
                                .unwrap_or(&"".to_string())
                        );
                        println!(
                            "  SV: {}",
                            sv_translation_file
                                .entries
                                .get(&id)
                                .unwrap_or(&"".to_string())
                        );
                    }
                }
            }
        }
    }
}

/// Find all usages of `<FormattedMessage />` in a file, and return
/// the id (e.g. `id="CO2_title"`) of the translation key used as
/// well as the line number.
fn find_formatted_message_usages(file_reader: BufReader<File>) -> Vec<(usize, String)> {
    let mut translation_key_usages: Vec<(usize, String)> = Vec::new();

    let mut looking_for_id = false;
    for (line_number, line) in file_reader.lines().enumerate() {
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
                }
                looking_for_id = false;
            }
            (Err(e), _) => {
                println!("Error reading line: {}", e);
            }
        }
    }

    translation_key_usages
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

fn is_node_modules(entry: &DirEntry) -> bool {
    entry.file_name() == "node_modules"
}
