use std::{collections::HashSet, path::PathBuf};

use clap::Parser;
use tl_check::{translation_file::TranslationFile, ts_file::TSFile};
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
    /// Path to key ignore unused file
    #[arg(short, long)]
    ignore_file: Option<PathBuf>,
}

static EXTENSIONS_TO_SEARCH: [&str; 2] = ["ts", "tsx"];

fn main() {
    let args = Args::parse();

    // Try to open the translation files
    let en_translation_file = TranslationFile::new(args.en_file);
    let sv_translation_file = TranslationFile::new(args.sv_file);
    if let Err((en_errors, sv_errors)) =
        en_translation_file.is_compatible_with(&sv_translation_file)
    {
        for error in en_errors {
            println!(
                "[{}]: {}",
                en_translation_file.path.to_str().unwrap(),
                error
            );
        }
        for error in sv_errors {
            println!(
                "[{}]: {}",
                sv_translation_file.path.to_str().unwrap(),
                error
            );
        }

        std::process::exit(1);
    }

    // Test against all TS files in the root directory
    let walker = WalkDir::new(args.root_dir)
        .into_iter()
        // Exclude node_modules
        .filter_entry(|e| !is_node_modules(e))
        // Filter out any non-accessible files
        .filter_map(|e| e.ok());

    let mut used_keys = HashSet::new();
    for entry in walker {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if EXTENSIONS_TO_SEARCH.contains(&ext.to_str().unwrap()) {
                    let mut ts_file = TSFile::new(path);

                    let formatted_message_keys = ts_file
                        .find_formatted_message_usages()
                        .into_iter()
                        .map(|(_, key)| key)
                        .collect::<Vec<_>>();
                    let format_message_keys = ts_file
                        .find_format_message_usages()
                        .into_iter()
                        .map(|(_, key)| key)
                        .collect::<Vec<_>>();
                    let misc_usages = ts_file
                        .find_misc_usages()
                        .into_iter()
                        .map(|(_, key)| key)
                        .collect::<Vec<_>>();

                    // Insert all keys into the set
                    used_keys.extend(format_message_keys);
                    used_keys.extend(formatted_message_keys);
                    used_keys.extend(misc_usages);
                }
            }
        }
    }

    // Check that all keys are used
    let ignore_unused_keys = if let Some(ignore_file) = args.ignore_file {
        let ignore_file = std::fs::read_to_string(ignore_file).unwrap();
        ignore_file
            .lines()
            .map(|line| line.trim().to_string())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut unused_keys = Vec::new();
    en_translation_file.entries.iter().for_each(|(key, value)| {
        if !used_keys.contains(key) && !ignore_unused_keys.contains(key) {
            println!("key \"{}\"=\"{}\" is never used!", key, value);
            unused_keys.push(key.clone());
        }
    });

    println!("{} unused keys", unused_keys.len());
    println!("{} used keys", used_keys.len());

    if !unused_keys.is_empty() {
        std::process::exit(1);
    }
}

fn is_node_modules(entry: &DirEntry) -> bool {
    entry.file_name() == "node_modules"
}
