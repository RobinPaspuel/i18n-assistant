use clap::{ Parser, Subcommand };
use dialoguer::{ Input, Select };
use glob::glob;
use std::fs;
use std::path::{ Path, PathBuf };
use std::process;
use indicatif::{ ProgressBar, ProgressStyle };

mod config;
mod sorter;
mod analyzer;

use config::{ Config, DictionaryFile, UsagePattern };
use sorter::sort_js_object;
use analyzer::{ analyze_file, collect_all_files };

#[derive(Parser)]
#[command(name = "i18na")]
#[command(about = "i18n Assistant Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure i18na for your project
    Configure,
    /// Sort translation files
    Sort {
        /// Sort a specific file by name (e.g., en_AR) or provide a full path (e.g., src/i18n/en/en_AR.js)
        #[arg(long, conflicts_with_all = &["language", "all"])]
        file: Option<String>,

        /// Sort all files within a specific language (e.g., en, es)
        #[arg(long, conflicts_with_all = &["file", "all"])]
        language: Option<String>,

        /// Sort all translation files across all languages
        #[arg(long, conflicts_with_all = &["file", "language"])]
        all: bool,

        /// Enable verbose logging
        #[arg(long)]
        verbose: bool,
    },
    /// Analyze i18n usages in your project
    Analyze {
        /// Analyze a specific file by name or provide a full path
        #[arg(long, conflicts_with_all = &["all"])]
        file: Option<String>,

        /// Analyze all translation files across all languages
        #[arg(long, conflicts_with_all = &["file"])]
        all: bool,

        /// Enable verbose logging
        #[arg(long)]
        verbose: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Configure => configure(),
        Commands::Sort { file, language, all, verbose } =>
            sort_translations(file.clone(), language.clone(), *all, *verbose),
        Commands::Analyze { file, all, verbose } => analyze_usages(file.clone(), *all, *verbose),
    }
}

/// Handles the `configure` subcommand.
/// Guides the user through setting up the i18na tool and generates the `.i18narc` file.
fn configure() {
    println!("Welcome to i18na Configuration!");

    // 1. Get src path
    let src_path: String = Input::new()
        .with_prompt("Enter the path to your source directory (e.g., src)")
        .validate_with(
            |input: &String| -> Result<(), &str> {
                if Path::new(input).exists() {
                    Ok(())
                } else {
                    Err("Path does not exist. Please enter a valid path.")
                }
            }
        )
        .interact_text()
        .unwrap();

    // 2. Get i18n path
    let i18n_path: String = Input::new()
        .with_prompt("Enter the path to your i18n directory (e.g., src/i18n)")
        .validate_with(
            |input: &String| -> Result<(), &str> {
                if Path::new(input).exists() {
                    Ok(())
                } else {
                    Err("Path does not exist. Please enter a valid path.")
                }
            }
        )
        .interact_text()
        .unwrap();

    // 3. Get dictionary file configuration
    let mut file_extension: String = Input::new()
        .with_prompt("Enter the file extension for your dictionary files (e.g., js)")
        .default("js".to_string())
        .interact_text()
        .unwrap();

    // Normalize the file extension by removing any leading dots
    if file_extension.starts_with('.') {
        file_extension = file_extension.trim_start_matches('.').to_string();
    }

    let variable_name: String = Input::new()
        .with_prompt("Enter the variable name used for dictionaries (e.g., dictionary)")
        .default("dictionary".to_string())
        .interact_text()
        .unwrap();

    // 4. Get usage pattern
    let method_name: String = Input::new()
        .with_prompt("Enter the method name used to access translations (e.g., get)")
        .default("get".to_string())
        .interact_text()
        .unwrap();

    let arguments: Vec<String> = {
        println!("Define the arguments for the method '{}'.", method_name);
        let arg1: String = Input::new()
            .with_prompt("Enter the first argument name (e.g., key)")
            .interact_text()
            .unwrap();
        let arg2: String = Input::new()
            .with_prompt("Enter the second argument name (e.g., defaultValue)")
            .interact_text()
            .unwrap();
        vec![arg1, arg2]
    };

    // Create Config
    let config = Config {
        src_path,
        i18n_path,
        dictionary_file: DictionaryFile {
            file_extension,
            variable_name,
        },
        usage_pattern: UsagePattern {
            method_name,
            arguments,
        },
    };

    // Serialize to JSON
    let config_json = serde_json::to_string_pretty(&config).unwrap();

    // Write to .i18narc
    fs::write(".i18narc", config_json).expect("Unable to write .i18narc");

    println!("\nConfiguration saved to .i18narc");
}

/// Handles the `sort` subcommand.
/// Sorts translation files based on the provided flags or through interactive prompts.
fn sort_translations(file: Option<String>, language: Option<String>, all: bool, verbose: bool) {
    let config = load_config();

    if file.is_some() || language.is_some() || all {
        // Handle non-interactive sorting based on flags
        if let Some(file_arg) = file {
            let file_path = construct_file_path(&file_arg, &config);
            sort_specific_file(&file_path, &config, verbose);
        } else if let Some(lang) = language {
            sort_language(&config, &lang, verbose);
        } else if all {
            sort_all_languages(&config, verbose);
        }
    } else {
        // Handle interactive sorting when no flags are provided
        sort_interactive(&config, verbose);
    }

    if !verbose {
        println!("Sorting completed successfully.");
    }
}

/// Handles the `analyze` subcommand.
fn analyze_usages(file: Option<String>, all: bool, verbose: bool) {
    let config = load_config();

    // Determine which files to analyze based on parameters
    let files_to_analyze = if let Some(file_arg) = file {
        vec![construct_file_path(&file_arg, &config)]
    } else if all {
        // Analyze all translation files across all languages
        collect_all_files(&config)
    } else {
        // Interactive mode
        analyze_interactive(&config)
    };

    let total_files = files_to_analyze.len();
    let mut total_issues = 0;

    // Initialize progress bar
    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .progress_chars("#>-")
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})"
            )
            .expect("Failed to set progress bar template")
    );

    for file_path in files_to_analyze {
        if verbose {
            println!("Analyzing file: {}", file_path);
        }

        match analyze_file(&file_path) {
            Ok(issues) => {
                if !issues.is_empty() {
                    total_issues += issues.len();
                    for issue in issues {
                        println!(
                            "Error: {}\nFile: {}\nLine: {}\nSuggestion: {}\n",
                            issue.description,
                            issue.file,
                            issue.line,
                            issue.suggestion
                        );
                    }
                }
            }
            Err(e) => {
                println!("Failed to analyze file '{}': {}", file_path, e);
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message("Analysis complete.");

    println!("\nAnalysis Summary:");
    println!("Total Files Analyzed: {}", total_files);
    println!("Total Issues Found: {}", total_issues);

    if total_issues == 0 {
        println!("No issues found in i18n usages.");
    } else {
        std::process::exit(1); // Exit with error code if issues are found
    }
}

/// Implements interactive analysis for the `analyze` subcommand.
/// Allows users to navigate directories and select files for analysis.
fn analyze_interactive(config: &Config) -> Vec<String> {
    let mut selected_files = Vec::new();
    let mut current_path = Path::new(&config.src_path).to_path_buf();

    loop {
        // List directories and .js files in the current directory
        let (directories, js_files) = list_dir_contents(&current_path);

        // Prepare items for selection
        let mut items = Vec::new();
        for dir in &directories {
            items.push(format!("{}/", Path::new(dir).display()));
        }
        for file in &js_files {
            items.push(file.clone());
        }

        // Add navigation options
        if current_path != Path::new(&config.src_path) {
            items.push(".. (Go Up)".to_string());
        }
        items.push("Finish Selection".to_string());

        // Prompt user to select one item
        let selection = Select::new()
            .with_prompt(format!("Current Directory: {}", current_path.display()))
            .default(0)
            .items(&items)
            .interact()
            .unwrap();

        let selected_item = &items[selection];
        if selected_item == ".. (Go Up)" {
            current_path = current_path.parent().unwrap().to_path_buf();
            continue;
        } else if selected_item == "Finish Selection" {
            break;
        }

        let selected_path = current_path.join(selected_item.trim_end_matches('/'));

        if selected_item.ends_with('/') {
            // It's a directory; navigate into it
            current_path = selected_path;
        } else {
            // It's a file; add to the list
            selected_files.push(selected_path.to_str().unwrap().to_string());
        }
    }

    selected_files
}

/// Lists directories and `.js` files in the given path.
///
/// Returns a tuple containing a vector of directories and a vector of `.js` files.
fn list_dir_contents(path: &PathBuf) -> (Vec<String>, Vec<String>) {
    let mut directories = Vec::new();
    let mut js_files = Vec::new();

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries.filter_map(Result::ok) {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if let Some(dir_name) = entry_path.file_name() {
                        directories.push(dir_name.to_string_lossy().into_owned());
                    }
                } else if entry_path.is_file() {
                    if let Some(ext) = entry_path.extension() {
                        if ext == "js" {
                            if let Some(file_name) = entry_path.file_name() {
                                js_files.push(file_name.to_string_lossy().into_owned());
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading directory '{}': {}", path.display(), e);
        }
    }

    (directories, js_files)
}

/// Constructs the full file path based on the input.
/// If the input is a full path, it uses it as-is (appending extension if missing).
/// If the input is a file name, it prepends the i18n_path and appends the extension if missing.
fn construct_file_path(file_arg: &str, config: &Config) -> String {
    let file_path = Path::new(file_arg);
    if file_path.is_absolute() || file_arg.contains(std::path::MAIN_SEPARATOR) {
        // Assume it's a path
        if file_path.extension().is_none() {
            format!("{}.{}", file_arg, config.dictionary_file.file_extension)
        } else {
            file_arg.to_string()
        }
    } else {
        // Assume it's a file name without path
        format!("{}/{}.{}", config.i18n_path, file_arg, config.dictionary_file.file_extension)
    }
}

/// Loads the configuration from `.i18narc`.
fn load_config() -> Config {
    let config_content = fs
        ::read_to_string(".i18narc")
        .expect("Failed to read .i18narc. Please run `i18na configure` first.");
    serde_json::from_str(&config_content).expect("Invalid .i18narc format.")
}

/// Sorts a specific translation file by its full path.
fn sort_specific_file(file_path: &str, config: &Config, verbose: bool) {
    if !Path::new(&file_path).exists() {
        println!("File '{}' does not exist.", file_path);
        return;
    }

    if verbose {
        println!("Sorting file: {}", file_path);
    }

    let content = fs::read_to_string(&file_path).expect("Failed to read file.");
    let sorted_content = sort_js_object(
        &content,
        &config.dictionary_file.variable_name
    ).unwrap_or_else(|e| {
        println!("Error sorting file '{}': {}", file_path, e);
        process::exit(1);
    });

    fs::write(&file_path, sorted_content).expect("Failed to write sorted content.");

    if verbose {
        println!("File '{}' sorted successfully.", file_path);
    }
}

/// Sorts all translation files within a specific language directory.
fn sort_language(config: &Config, language: &str, verbose: bool) {
    let language_path = format!("{}/{}", config.i18n_path, language);

    if !Path::new(&language_path).is_dir() {
        println!("Language directory '{}' does not exist.", language_path);
        return;
    }

    if verbose {
        println!("Sorting all files in language: {}", language);
    }

    let pattern = format!(
        "{}/{}/*.{}",
        config.i18n_path,
        language,
        config.dictionary_file.file_extension
    );

    if verbose {
        println!("Using glob pattern: {}", pattern);
    }

    for entry in glob(&pattern).expect("Failed to read glob pattern.") {
        match entry {
            Ok(path) => {
                let file_path = path.to_str().unwrap();
                sort_specific_file(file_path, config, verbose);
            }
            Err(e) => println!("Error reading path: {:?}", e),
        }
    }
}

/// Sorts all translation files across all language directories.
fn sort_all_languages(config: &Config, verbose: bool) {
    if verbose {
        println!("Sorting all translation files across all languages.");
    }

    let languages = fs
        ::read_dir(&config.i18n_path)
        .expect("Failed to read i18n directory.")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().into_string().unwrap())
        .collect::<Vec<String>>();

    if languages.is_empty() {
        println!("No language folders found in '{}'.", config.i18n_path);
        return;
    }

    for language in languages {
        sort_language(config, &language, verbose);
    }
}

/// Handles interactive sorting when no flags are provided.
fn sort_interactive(config: &Config, verbose: bool) {
    // 1. List language directories
    let languages = fs
        ::read_dir(&config.i18n_path)
        .expect("Failed to read i18n directory.")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().into_string().unwrap())
        .collect::<Vec<String>>();

    if languages.is_empty() {
        println!("No language folders found in '{}'.", config.i18n_path);
        return;
    }

    // 2. Let the user select a language
    let selection = Select::new()
        .with_prompt("Select the language directory to sort")
        .default(0)
        .items(&languages)
        .interact()
        .unwrap();

    let selected_language = &languages[selection];
    if verbose {
        println!("Selected language: {}", selected_language);
    }

    // 3. List translation files in the selected language directory
    let language_path = format!("{}/{}", config.i18n_path, selected_language);
    let pattern = format!(
        "{}/{}/*.{}",
        config.i18n_path,
        selected_language,
        config.dictionary_file.file_extension
    );

    if verbose {
        println!("Using glob pattern: {}", pattern);
    }

    let files = glob(&pattern)
        .expect("Failed to read glob pattern.")
        .filter_map(Result::ok)
        .map(|path| path.file_stem().unwrap().to_string_lossy().into_owned())
        .collect::<Vec<String>>();

    if files.is_empty() {
        println!(
            "No files with extension '{}' found in '{}'.",
            config.dictionary_file.file_extension,
            language_path
        );
        return;
    }

    // 4. Let the user select a file to sort
    let file_selection = Select::new()
        .with_prompt("Select the translation file to sort")
        .default(0)
        .items(&files)
        .interact()
        .unwrap();

    let selected_file = &files[file_selection];
    if verbose {
        println!("Selected file: {}", selected_file);
    }

    // 5. Construct the full file path
    let file_path = format!(
        "{}/{}.{}",
        language_path,
        selected_file,
        config.dictionary_file.file_extension
    );

    // 6. Sort the selected file
    sort_specific_file(&file_path, config, verbose);
}
