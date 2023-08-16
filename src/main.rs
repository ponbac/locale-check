use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use clap::Parser;
use walkdir::{DirEntry, WalkDir};

/// Handle those damn translations...
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Root directory to search from
    #[arg(short, long, default_value = ".")]
    path: PathBuf,
}

static EXTENSIONS_TO_SEARCH: [&str; 2] = ["ts", "tsx"];

fn main() {
    let args = Args::parse();

    let walker = WalkDir::new(args.path)
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
                            "[{}] Found key: {} on line {}",
                            path.file_name().unwrap().to_str().unwrap(),
                            id,
                            line_number
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
                }

                if line.contains("id=") {
                    let key = line
                        .split("id=")
                        .nth(1)
                        .unwrap()
                        .split('"')
                        .nth(1)
                        .unwrap()
                        .to_string();
                    translation_key_usages.push((line_number, key));
                    looking_for_id = false;
                }
            }
            (Ok(line), true) => {
                if line.contains("id=") {
                    let key = line
                        .split("id=")
                        .nth(1)
                        .unwrap()
                        .split('"')
                        .nth(1)
                        .unwrap()
                        .to_string();
                    translation_key_usages.push((line_number, key));
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

fn is_node_modules(entry: &DirEntry) -> bool {
    entry.file_name() == "node_modules"
}
