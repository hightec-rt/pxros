//! 
//! SPDX-FileCopyrightText: Veecle GmbH, HighTec EDV-Systeme GmbH
//! 
//! SPDX-License-Identifier: Apache-2.0
//! 
use std::fs::File;
use std::io::Read;

use serde_json::{self, json, Value};

/// Transforms the input JSON file specified by `file_path`.
///
/// This function opens and reads a JSON file, then parses its contents to perform transformations on specific fields
/// within the JSON object. It modifies the `description` field by concatenating text elements and formatting bullet
/// lists. Additionally, it restructures the `appliesTo` and `errCodes` fields by aggregating their items under a
/// specified key within these arrays. These modifications are designed to reformat the JSON content for enhanced
/// processing or display.
///
/// # Parameters
///
/// - `file_path`: A string slice that holds the path to the JSON file to be transformed.
///
/// # Returns
///
/// Returns a `String` containing the pretty-printed JSON after the transformations have been applied.
///
/// # Panics
///
/// This function will panic if:
/// - The file specified by `file_path` cannot be opened.
/// - The file contents cannot be read into a string.
/// - The file content is not valid JSON or cannot be parsed as such.
/// - The transformed JSON object cannot be serialized back into a string format.
///
/// # Examples
///
/// ```
/// let transformed_json = transform_input("/path/to/input.json");
/// println!("{}", transformed_json);
/// ```
///
/// Note: Replace `"/path/to/input.json"` with the actual path to your JSON file.
pub fn transform_input(file_path: &str) -> String {
    let mut file = File::open(file_path).expect("Failed to open file");

    // Read the file contents into a string
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read file");

    // Parse the JSON string into a serde_json::Value
    let mut json_value: Value = serde_json::from_str(&contents).expect("Failed to parse JSON");

    // Generic function to modify structures
    /// Modifies a JSON structure based on a specified key and target key.
    ///
    /// This function takes a mutable reference to a JSON value, a key, and a target key. It first checks if the JSON
    /// value contains an array associated with the provided key. If such an array exists, the function checks if the
    /// first entry of the array contains the target key.
    ///
    /// If the target key is found, the function retrieves the value associated with the target key from the first entry
    /// of the array. Depending on the type of the target value, the function performs different operations:
    /// - If the target value is an array, the function collects all its values and replaces the original array with the
    ///   collected values.
    /// - If the target value is a string, the function wraps the string in a new array and replaces the original array
    ///   with this new array.
    ///
    /// If the target key does not exist but the array contains an entry with the key "ARM-CMX", the function replaces
    /// the original array with an empty array.
    ///
    /// If the target key is not found in the first entry of the array and there is no entry with the key "ARM-CMX", the
    /// function simply clones the original array.
    ///
    /// # Arguments
    ///
    /// * `json_value` - A mutable reference to the JSON value being modified.
    /// * `key` - The key within the JSON value to target for modification.
    /// * `target_key` - The target key within the first entry of the array to look for.
    fn modify_structure(json_value: &mut Value, key: &str, target_key: &str) {
        if let Some(array) = json_value.get_mut(key).and_then(Value::as_array_mut) {
            let target_value = array.first().and_then(|entry| entry.get(target_key));
            *array = match target_value {
                Some(target_array) if target_array.is_array() => target_array.as_array().unwrap().to_vec(),
                Some(target_string) if target_string.is_string() => {
                    vec![json!(target_string.as_str().unwrap())]
                },
                _ if array.iter().any(|entry| entry.get("ARM-CMX").is_some()) => Vec::new(),
                _ => array.clone(),
            };
        }
    }

    // Transforms the "long" part of the description within a JSON value into a concatenated string format.
    /// Transforms the "long" description within a JSON object based on a target key.
    ///
    /// This function iterates over the "long" field of a "description" object within the JSON,
    /// transforming its content based on the provided target key. It handles different types of
    /// content, such as paragraphs ("PP") and bullet lists ("BL"), and restructures them into a
    /// single string. The transformed text is then updated back into the "long" field as a single
    /// array element.
    ///
    /// # Arguments
    ///
    /// * `json_value` - A mutable reference to the JSON value being transformed.
    /// * `target_key` - The key within the "long" description to target for transformation.
    fn transform_description(json_value: &mut Value, target_key: &str) {
        if let Some(description) = json_value.get_mut("description") {
            if let Some(long) = description.get_mut("long").and_then(|l| l.as_array_mut()) {
                let mut transformed_text = String::new();

                for item in long.iter() {
                    if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                        match item_type {
                            "PP" => {
                                if let Some(pp_content) = item.get(target_key).and_then(|v| v.as_array()) {
                                    for content in pp_content {
                                        if let Some(content_text) = content.as_str() {
                                            transformed_text.push('\n');
                                            transformed_text.push_str(content_text);
                                            transformed_text.push('\n');
                                        }
                                    }
                                } else if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                    transformed_text.push('\n');
                                    transformed_text.push_str(text);
                                    transformed_text.push('\n');
                                }
                            },
                            "BL" => {
                                if let Some(bullets) = item.get(target_key).and_then(|v| v.as_array()) {
                                    for bullet in bullets {
                                        if let Some(bullet_text) = bullet.as_str() {
                                            transformed_text.push_str("\n * ");
                                            transformed_text.push_str(bullet_text);
                                        }
                                    }
                                } else if let Some(bullets) = item.get("text").and_then(|t| t.as_array()) {
                                    for bullet in bullets {
                                        if let Some(bullet_text) = bullet.as_str() {
                                            transformed_text.push_str("\n * ");
                                            transformed_text.push_str(bullet_text);
                                        }
                                    }
                                    transformed_text.push('\n');
                                }
                            },
                            _ => {},
                        }
                    }
                }

                // Update the description.long to a single array element with the transformed text
                *long = vec![json!({
                    "type": "PP",
                    "text": transformed_text.trim().to_string()
                })];
            }
        }
    }

    /// Transforms the "cop" sections of a JSON value based on a target key.
    ///
    /// This function iterates over specified sections within the "cop" field of a JSON object,
    /// transforming each section by extracting and restructuring text based on the provided target key.
    /// The sections considered for transformation include "BeforeCall", "AfterCall", and "BestPractice".
    ///
    /// # Arguments
    ///
    /// * `json_value` - A mutable reference to the JSON value being transformed.
    /// * `target_key` - The key within each section to target for transformation.
    fn transform_cop(json_value: &mut Value, target_key: &str) {
        if let Some(cop) = json_value.get_mut("cop") {
            for key in ["BeforeCall", "AfterCall", "BestPractice"].iter() {
                if let Some(section) = cop.get_mut(*key).and_then(|s| s.as_array_mut()) {
                    if let Some(first) = section.first_mut() {
                        if let Some(target_array) = first.get_mut(target_key).and_then(|ta| ta.as_array_mut()) {
                            let mut new_section = Vec::new();
                            for item in target_array.iter() {
                                if let Some(text) = item.as_str() {
                                    new_section.push(json!(text));
                                }
                            }
                            *section = new_section;
                        }
                    }
                }
            }
        }
    }

    // Apply the generic function to modify `appliesTo`, `errCodes`, and `usage` structures
    modify_structure(&mut json_value, "synopsis", "TC23");
    modify_structure(&mut json_value, "arguments", "TC23");
    modify_structure(&mut json_value, "appliesTo", "TC23");
    modify_structure(&mut json_value, "errCodes", "TC23");
    modify_structure(&mut json_value, "usage", "TC23");
    transform_description(&mut json_value, "TC23");
    transform_cop(&mut json_value, "TC23");

    serde_json::to_string_pretty(&json_value).expect("[*] =====> Failed to transform input file !!!")
}
