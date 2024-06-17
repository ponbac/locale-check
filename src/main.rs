use std::path::PathBuf;

use clap::Parser;
use console::style;
use ramilang::{
    interactive,
    translation_file::{TranslationFile, TranslationFileError},
    ts_file::{KeyUsage, TSFile},
};
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
    #[arg(long)]
    ignore_file: Option<PathBuf>,
    /// Sort keys in translation files
    #[arg(long, action)]
    sort: bool,
    /// Interactive mode
    #[arg(long, short, action)]
    interactive: bool,
}

static EXTENSIONS_TO_SEARCH: [&str; 2] = ["ts", "tsx"];

// clear; cargo run -- --sort --root-dir C:\Users\pbac\dev\SE-CustomerPortal\CustomerPortal\ --en-file C:\Users\pbac\dev\SE-CustomerPortal\CustomerPortal\shared\translations\en.json --sv-file C:\Users\pbac\dev\SE-CustomerPortal\CustomerPortal\shared\translations\sv.json --ignore-file C:\Users\pbac\dev\SE-CustomerPortal\CustomerPortal\shared\translations\.keyignore
#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Try to open the translation files
    let en_translation_file = TranslationFile::new(args.en_file.clone());
    let sv_translation_file = TranslationFile::new(args.sv_file.clone());

    println!("\n{}\n", style("Checking translations...").blue().bold());
    // Check for problems with the translation files
    match (&en_translation_file, &sv_translation_file) {
        (Err(err), _) | (_, Err(err)) => {
            println!(
                "{}{}",
                style("ERROR").red().bold(),
                style(format!(": {}", err)).bold()
            );
            std::process::exit(1);
        }
        (Ok(en_translation_file), Ok(sv_translation_file)) => {
            if let Err((en_errors, sv_errors)) =
                en_translation_file.is_compatible_with(sv_translation_file)
            {
                for error in en_errors.iter().chain(sv_errors.iter()) {
                    match error {
                        TranslationFileError::MissingKey { key, missing_in } => {
                            println!(
                                "{} key {} not found in {}",
                                style("[MISSING]").yellow().bold(),
                                style(key).bold(),
                                style(missing_in.to_str().unwrap()).italic()
                            );
                        }
                        TranslationFileError::EmptyValue(key) => {
                            println!(
                                "{} key {} seems to be empty",
                                style("[EMPTY]").yellow().bold(),
                                style(key).bold()
                            );
                        }
                        _ => {}
                    }
                }

                println!(
                    "{}{}",
                    style("ERROR").red().bold(),
                    style(": translation files are not compatible, see problems above").bold()
                );
                std::process::exit(1);
            }
        }
    }

    // Test against all TS files in the root directory
    let walker = WalkDir::new(args.root_dir)
        .into_iter()
        // Exclude node_modules
        .filter_entry(|e| !is_node_modules(e))
        // Filter out any non-accessible files
        .filter_map(|e| e.ok());

    let mut key_usages: Vec<KeyUsage> = Vec::new();
    for file in walker.filter(|e| e.path().is_file()) {
        if let Some(ext) = file.path().extension() {
            if EXTENSIONS_TO_SEARCH.contains(&ext.to_str().unwrap()) {
                let mut ts_file = TSFile::new(file.path());

                // Collect KeyUsage from different methods
                let formatted_message_keys = ts_file.find_formatted_message_usages();
                let format_message_keys = ts_file.find_format_message_usages();
                let misc_usages = ts_file.find_misc_usages();

                // Extend the used_keys vector with the KeyUsages
                key_usages.extend(format_message_keys);
                key_usages.extend(formatted_message_keys);
                key_usages.extend(misc_usages);
            }
        }
    }

    // Check that all usages are valid
    let mut n_invalid_usages = 0;

    let entries = en_translation_file.as_ref().unwrap().entries.clone();
    key_usages.iter().for_each(|usage| {
        if !entries.contains_key(usage.key.as_str()) {
            println!(
                "{} key {} does not exist! {}",
                style("[INVALID]").yellow().bold(),
                style(usage.key.as_str()).bold(),
                style(format!(
                    "({}:{})",
                    usage.file_path.to_str().unwrap(),
                    usage.line
                ))
                .italic()
                .dim()
            );
            n_invalid_usages += 1;
        }
    });

    if n_invalid_usages != 0 {
        println!(
            "{}{}",
            style("ERROR").red().bold(),
            style(format!(": {} invalid key usages!", n_invalid_usages)).bold(),
        );
        std::process::exit(1);
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
    en_translation_file
        .unwrap()
        .entries
        .iter()
        .for_each(|(key, value)| {
            if !key_usages.iter().any(|usage| usage.key == *key)
                && !ignore_unused_keys.contains(key)
            {
                println!(
                    "{} key {}={}",
                    style("[UNUSED]").yellow().bold(),
                    style(key).bold(),
                    style(format!("\"{}\"", value)).italic(),
                );
                unused_keys.push(key.clone());
            }
        });

    if !unused_keys.is_empty() {
        println!(
            "{}{} {}",
            style("ERROR").red().bold(),
            style(format!(": {} unused keys found!", unused_keys.len(),)).bold(),
            style(format!("({} keys ignored)", ignore_unused_keys.len())).italic()
        );
        println!(
            "{}",
            style("Unused keys should be removed from the translation files if they really are unused.").italic()
        );
        println!(
            "{}",
            style(
                "If they are used (false positive), add them to the ignore file (--ignore-file)."
            )
            .italic()
        );
        std::process::exit(1);
    }

    println!(
        "{}{}",
        style("SUCCESS").green().bold(),
        style(": great translations!").bold()
    );

    // Sort the translation files if requested (should maybe always be done?)
    if args.sort {
        println!(
            "\n{}\n",
            style("Sorting translation files...").blue().bold()
        );

        let en_file = TranslationFile::new(args.en_file.clone()).unwrap();
        let sv_file = TranslationFile::new(args.sv_file.clone()).unwrap();
        // The translation files are sorted by default (BTreeMap), so we just need to write them back
        en_file.write().expect("Unable to write EN file");
        sv_file.write().expect("Unable to sort SV file");

        println!(
            "{}{}",
            style("SUCCESS").green().bold(),
            style(": translation files sorted!").bold()
        );
    }

    if args.interactive {
        println!(
            "\n{}\n",
            style("Starting interactive server...").blue().bold()
        );
        let _ = interactive::run_server(args.en_file.as_path(), args.sv_file.as_path()).await;
    }
}

fn is_node_modules(entry: &DirEntry) -> bool {
    entry.file_name() == "node_modules"
}
