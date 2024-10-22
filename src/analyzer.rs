use regex::Regex;
use std::fs;
use std::path::Path;

/// Represents an issue found during analysis.
pub struct Issue {
    pub description: String,
    pub file: String,
    pub line: usize,
    pub suggestion: String,
}

/// Analyzes a single file for improper i18n usages.
/// Returns a vector of `Issue` if any are found.
pub fn analyze_file(file_path: &str) -> Result<Vec<Issue>, String> {
    let content = fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let mut issues = Vec::new();

    // Updated regex without backreferences using alternation for single and double quotes
    let re = Regex::new(
        r#"I18n\.get\s*\(\s*'([^']*)'\s*,\s*'([^']*)'\s*\)|I18n\.get\s*\(\s*"([^"]*)"\s*,\s*"([^"]*)"\s*\)"#
    ).map_err(|e| format!("Failed to compile regex: {}", e))?;

    for (line_number, line) in content.lines().enumerate() {
        for caps in re.captures_iter(line) {
            // Extract key and default_value based on which pattern matched
            let (key, default_value) = if let Some(k) = caps.get(1) {
                // Single quotes matched
                (k.as_str(), caps.get(2).map_or("", |m| m.as_str()))
            } else if let Some(k) = caps.get(3) {
                // Double quotes matched
                (k.as_str(), caps.get(4).map_or("", |m| m.as_str()))
            } else {
                ("", "")
            };

            let key = key.trim();
            let default_value = default_value.trim();

            if key.is_empty() && !default_value.is_empty() {
                issues.push(Issue {
                    description: "Missing key in I18n.get usage.".to_string(),
                    file: file_path.to_string(),
                    line: line_number + 1,
                    suggestion: "Provide a valid key for the I18n.get method.".to_string(),
                });
            }

            if !key.is_empty() && default_value.is_empty() {
                issues.push(Issue {
                    description: "Missing default value in I18n.get usage.".to_string(),
                    file: file_path.to_string(),
                    line: line_number + 1,
                    suggestion: "Add a default value to the I18n.get method.".to_string(),
                });
            }

            if key.is_empty() && default_value.is_empty() {
                issues.push(Issue {
                    description: "Missing both key and default value in I18n.get usage.".to_string(),
                    file: file_path.to_string(),
                    line: line_number + 1,
                    suggestion: "Provide both a key and a default value for the I18n.get method.".to_string(),
                });
            }
        }
    }

    Ok(issues)
}

/// Collects all relevant `.js` files across all languages based on the configuration.
pub fn collect_all_files(config: &super::config::Config) -> Vec<String> {
    let mut files = Vec::new();

    let src_path = Path::new(&config.src_path);
    let pattern = format!("{}/**/*.js", src_path.display());

    for entry in glob::glob(&pattern).expect("Failed to read glob pattern.") {
        match entry {
            Ok(path) => {
                if let Some(path_str) = path.to_str() {
                    files.push(path_str.to_string());
                }
            }
            Err(e) => println!("Error reading path: {:?}", e),
        }
    }

    files
}
