use std::collections::BTreeMap;
use regex::Regex;

/// Sorts the keys of a JavaScript dictionary object alphabetically while preserving non-JSON lines.
///
/// # Arguments
///
/// * `content` - The content of the JavaScript file as a string.
/// * `variable_name` - The name of the variable that holds the dictionary.
///
/// # Returns
///
/// A new string with the dictionary keys sorted alphabetically and non-JSON lines preserved.
pub fn sort_js_object(content: &str, variable_name: &str) -> Result<String, String> {
    // Step 1: Find the start of the dictionary declaration
    let dict_declaration = format!("const {} = {{", variable_name);
    let dict_start = content
        .find(&dict_declaration)
        .ok_or_else(|| "Dictionary declaration not found.".to_string())?;
    let dict_start_brace = dict_start + dict_declaration.len() - 1; // Position of the opening brace `{`

    // Step 2: Find the end of the dictionary by looking for the last closing brace `}`
    let dict_end = content.rfind('}').ok_or_else(|| "Closing brace not found.".to_string())?;

    // Step 3: Extract the dictionary content
    let dict_content = &content[dict_start_brace + 1..dict_end].to_string();

    // Parse the dictionary content into spread lines and key-value pairs
    let (spread_lines, kv_map) = parse_dictionary(dict_content)?;

    // Serialize the key-value map to JSON
    let json_map: serde_json::Value = serde_json
        ::to_value(kv_map)
        .map_err(|e| format!("Failed to serialize key-value pairs to JSON: {}", e))?;

    // Serialize the sorted JSON map with pretty formatting
    let sorted_json = serde_json
        ::to_string_pretty(&json_map)
        .map_err(|e| format!("Failed to serialize sorted JSON: {}", e))?;

    // Reconstruct the dictionary by reinserting spread lines
    let reconstructed_dict = reconstruct_dictionary(
        &dict_declaration.to_string(),
        &spread_lines,
        &sorted_json.to_string()
    )?;

    // Extract the content before and after the dictionary
    let before = &content[..dict_start];
    let after = &content[dict_end + 1..];

    // Combine all parts
    Ok(format!("{}{}{}", before, reconstructed_dict, after))
}

/// Parses the dictionary content and returns spread lines and a BTreeMap of key-value pairs.
///
/// # Arguments
///
/// * `dict_content` - The content of the dictionary as a string.
///
/// # Returns
///
/// A tuple containing a vector of spread operator lines and a BTreeMap of key-value pairs.
fn parse_dictionary(dict_content: &str) -> Result<(Vec<String>, BTreeMap<String, String>), String> {
    let mut spread_lines = Vec::new();
    let mut kv_map = BTreeMap::new();

    // Regular expression to match key-value pairs
    // It captures the key and value, handling escaped quotes within values
    let re = Regex::new(
        r#"^\s*"([^"\\]*(?:\\.[^"\\]*)*)"\s*:\s*"([^"\\]*(?:\\.[^"\\]*)*)"\s*,?\s*$"#
    ).map_err(|e| format!("Failed to compile regex: {}", e))?;

    for line in dict_content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("...") {
            // Preserve spread operators
            spread_lines.push(trimmed.to_string());
            continue;
        }

        if trimmed.is_empty() {
            // Skip empty lines to prevent malformed output
            continue;
        }

        // Attempt to match key-value pairs
        if let Some(captures) = re.captures(trimmed) {
            let key = captures.get(1).unwrap().as_str().to_string();
            let value = captures.get(2).unwrap().as_str().to_string();
            kv_map.insert(key, value);
        } else {
            // If the line doesn't match, skip it to prevent malformed entries
            // Alternatively, you can log or handle it as needed
            continue;
        }
    }

    Ok((spread_lines, kv_map))
}

/// Reconstructs the sorted dictionary by reinserting spread lines.
///
/// # Arguments
///
/// * `dict_declaration` - The dictionary declaration string (e.g., "const dictionary = {").
/// * `spread_lines` - A vector of lines that are not key-value pairs (e.g., spread operators).
/// * `sorted_json` - The sorted JSON string.
///
/// # Returns
///
/// The reconstructed sorted dictionary as a string.
fn reconstruct_dictionary(
    dict_declaration: &str,
    spread_lines: &Vec<String>,
    sorted_json: &str
) -> Result<String, String> {
    // Initialize the reconstructed dictionary with the declaration (no extra '{')
    let mut reconstructed = format!("{}\n", dict_declaration);

    // Insert spread lines first without adding extra commas
    for spread in spread_lines {
        reconstructed.push_str(&format!("  {}\n", spread));
    }

    // Split the sorted_json into lines
    let json_lines: Vec<&str> = sorted_json.lines().collect();

    // Iterate through sorted_json lines and append key-value pairs
    for line in json_lines.iter().skip(1) {
        // skip the opening brace
        let trimmed = line.trim();
        reconstructed.push_str(&format!("  {}\n", trimmed));
    }

    Ok(reconstructed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_js_object_basic() {
        let content =
            r#"
import en from './en'

const dictionary = {
  ...en,
  "b_key": "Value B",
  "a_key": "Value A"
};

export default dictionary;
"#;

        let expected =
            r#"
import en from './en'

const dictionary = {
  ...en,
  "a_key": "Value A",
  "b_key": "Value B",
};

export default dictionary;
"#;

        let sorted = sort_js_object(content, "dictionary").unwrap();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_sort_js_object_with_braces_in_values() {
        let content =
            r#"
const dictionary = {
  ...en,
  "key_with_braces": "This is a value with {{braces}} inside.",
  "another_key": "Another value."
};

export default dictionary;
"#;

        let expected =
            r#"
const dictionary = {
  ...en,
  "another_key": "Another value.",
  "key_with_braces": "This is a value with {{braces}} inside.",
};

export default dictionary;
"#;

        let sorted = sort_js_object(content, "dictionary").unwrap();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_sort_js_object_missing_declaration() {
        let content = r#"
const dict = {
  "key": "value"
};

export default dict;
"#;

        let result = sort_js_object(content, "dictionary");
        assert!(result.is_err());
    }

    #[test]
    fn test_sort_js_object_with_apostrophe() {
        let content =
            r#"
const dictionary = {
  ...en,
  "greeting": "Hello, it's a nice day!",
  "farewell": "Goodbye!"
};

export default dictionary;
"#;

        let expected =
            r#"
const dictionary = {
  ...en,
  "farewell": "Goodbye!",
  "greeting": "Hello, it's a nice day!",
};

export default dictionary;
"#;

        let sorted = sort_js_object(content, "dictionary").unwrap();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_sort_js_object_with_curly_quotes() {
        let content =
            r#"
const dictionary = {
  ...en,
  "quote_test": "She said, “Hello, it's a wonderful day!”",
  "farewell": "Goodbye!"
};

export default dictionary;
"#;

        let expected =
            r#"
const dictionary = {
  ...en,
  "farewell": "Goodbye!",
  "quote_test": "She said, “Hello, it's a wonderful day!”",
};

export default dictionary;
"#;

        let sorted = sort_js_object(content, "dictionary").unwrap();
        assert_eq!(sorted, expected);
    }
}
