use std::path::PathBuf;

use clap::Parser;
use locale_check::{translation_file::TranslationFile, ts_file::TSFile};
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
                    let ts_file = TSFile::new(path);

                    let key_usages = ts_file.find_formatted_message_usages();
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

fn is_node_modules(entry: &DirEntry) -> bool {
    entry.file_name() == "node_modules"
}
