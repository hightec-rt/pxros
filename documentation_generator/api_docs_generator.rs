//!
//! SPDX-FileCopyrightText: Veecle GmbH, HighTec EDV-Systeme GmbH
//!
//! SPDX-License-Identifier: Apache-2.0
//!
use std::fmt;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Result;

use super::transform_input;

/// Represents the complete description of an API, including its name, arguments, return values, and associated
/// metadata.
#[derive(Serialize, Deserialize)]
pub struct ApiDescription {
    name: Name,
    synopsis: Option<Vec<String>>,
    arguments: Option<Vec<Argument>>,
    #[serde(rename = "retValues")]
    ret_values: Option<Vec<String>>,
    #[serde(rename = "errCodes")]
    err_codes: Option<Vec<String>>,
    #[serde(rename = "appliesTo")]
    applies_to: Vec<String>,
    description: Description,
    #[serde(rename = "seeAlso")]
    see_also: Option<Vec<SeeAlso>>,
    usage: Option<Vec<String>>,
    cop: Option<Cop>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Name {
    key: String,
    display: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Argument {
    name: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Description {
    short: String,
    long: Vec<Paragraph>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Paragraph {
    r#type: String,
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SeeAlso {
    key: String,
    display: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cop {
    #[serde(rename = "BeforeCall")]
    before_call: Vec<String>,
    #[serde(rename = "AfterCall")]
    after_call: Vec<String>,
    #[serde(rename = "BestPractice")]
    best_practice: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AppliesTo(Vec<Platform>);

#[derive(Serialize, Deserialize, Debug)]
struct Platform {
    #[serde(rename = "TC23")]
    tc23: Vec<String>,
    #[serde(rename = "ARM-CMX")]
    arm_cmx: Vec<String>,
}

/// Encapsulates sequences of capital letters with backticks for markdown formatting.
fn make_literal(input: &str) -> String {
    // Create a Regex pattern to match sequences of capital letters possibly interspersed with underscores
    let regex = Regex::new(r"\b[A-Z_]+\b").unwrap();

    // Replace matches in the string by encapsulating them with backticks
    regex.replace_all(input, "`$0`").into_owned()
}

fn remove_new_lines(line: &str) -> String {
    if line.starts_with('\n') || line.ends_with('\n') {
        line.to_string()
    } else {
        line.replace('\n', " ")
    }
}

/// Converts a C function signature to a Rust function signature.
///
/// This function takes a string representation of a C function signature as input, and returns a string representation
/// of the equivalent Rust function signature.
///
/// The function first trims any leading or trailing spaces from the input string. It then uses a regular expression to
/// split the string into its constituent parts: the return type, the function name, and the arguments.
///
/// The arguments are further processed to handle different types of parameters, including those with multiple words
/// (like `unsigned int`). The function also handles the special case where the C function has no parameters (i.e.,
/// `void`).
///
/// Finally, the function constructs the Rust function signature by joining the processed parts together in the correct
/// format, and returns this as a string.
///
/// # Arguments
///
/// * `c_func`: A string slice representing the C function signature to be converted.
///
/// # Returns
///
/// A `String` representing the equivalent Rust function signature.
fn convert_c_func_to_rust(c_func: &str) -> String {
    let trimmed_func = c_func.trim();
    let regex = Regex::new(r"\(([^)]*)\)").unwrap();
    let parts: Vec<&str> = trimmed_func.split_whitespace().collect();
    let return_type = parts[0];
    let func_name = parts[1].split('(').next().unwrap();
    let mut rust_params = Vec::new();
    let mut arguments: Vec<&str> = Vec::new();

    if let Some(caps) = regex.captures(trimmed_func) {
        if let Some(content) = caps.get(1) {
            let arguments_str = content.as_str();
            arguments = arguments_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    if !arguments.is_empty() && arguments[0] != "void" {
        for arg in &arguments {
            let p: Vec<&str> = arg
                .split(|c: char| (c == ',' || c.is_whitespace()))
                .filter(|s| !s.is_empty())
                .collect();
            match p.len() {
                2 => {
                    let (param_type, param_name) = (p[0].trim(), p[1].trim());
                    rust_params.push(format!("{}: {}", param_name, param_type));
                }
                3 => {
                    let (param_type, param_name, param_type2) =
                        (p[0].trim(), p[1].trim(), p[2].trim());
                    rust_params.push(format!("{}: {} {}", param_name, param_type, param_type2));
                }
                _ => (),
            }
        }
    }

    let rust_return_type = if return_type != "void" {
        format!("-> {}", return_type)
    } else {
        String::new()
    };

    let rust_params_str = rust_params.join(", ");

    format!(
        "fn {}({}) {};",
        func_name, rust_params_str, rust_return_type
    )
}

/// Writes a documentation section with a given title and items list, formatted according to `format_type`.
fn write_section<T, I>(
    f: &mut fmt::Formatter,
    title: &str,
    items: I,
    format_type: FormatType,
) -> fmt::Result
where
    T: AsRef<str>,
    I: IntoIterator<Item = T>,
{
    writeln!(f, "///")?;
    writeln!(f, "/// ### {}", title)?;

    match format_type {
        FormatType::Normal => {
            for item in items {
                writeln!(f, "/// {}", convert_c_func_to_rust(item.as_ref()))?;
                // writeln!(f, "/// {}", item.as_ref())?;
            }
        }
        FormatType::List { literal } => {
            for item in items {
                if literal {
                    writeln!(f, "/// * {}", make_literal(item.as_ref()))?;
                } else {
                    writeln!(f, "/// * {}", item.as_ref())?;
                }
            }
        }
        FormatType::Code => {
            writeln!(f, "/// ```c")?;
            for item in items {
                writeln!(f, "/// {}", item.as_ref().replace('\t', " "))?;
            }
            writeln!(f, "/// ```")?;
        }
    }

    Ok(())
}

/// Specifies the formatting type for documentation sections.
enum FormatType {
    #[allow(dead_code)]
    Normal,
    List {
        literal: bool,
    },
    Code,
}

/// Provides functionality for parsing a JSON string into an `ApiDescription` and formatting it for documentation.
impl ApiDescription {
    // Function to read from a JSON file and parse into the ApiDescription struct
    pub fn from_modified_string(json_string: &str) -> Result<Self> {
        // Parse the JSON into a ApiDescription.
        serde_json::from_str(json_string)
    }
}

/// Implements custom formatting for the `ApiDescription` struct, suitable for generating documentation comments.
impl fmt::Display for ApiDescription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Description
        writeln!(f, "/// {}", self.description.short)?;
        for paragraph in &self.description.long {
            writeln!(f, "///")?;
            for line in paragraph.text.split('\n') {
                writeln!(f, "/// {}", remove_new_lines(line))?;
            }
        }

        // Applies to version
        if !self.applies_to.is_empty() {
            write_section(
                f,
                "Applies To",
                self.applies_to.iter(),
                FormatType::List { literal: false },
            )?;
        }

        // Synopsis
        if let Some(synopsis) = &self.synopsis {
            if !synopsis.is_empty() {
                write_section(f, "Synopsis", synopsis.iter(), FormatType::Normal)?;
            }
        }

        // Arguments
        if let Some(arguments) = &self.arguments {
            if !arguments.is_empty() {
                writeln!(f, "///")?;
                writeln!(f, "/// ### Arguments")?;
                for arg in arguments {
                    writeln!(f, "/// * `{}`: {}", arg.name, arg.description)?;
                }
            }
        }

        // Return values
        if let Some(ret_values) = &self.ret_values {
            if !ret_values.is_empty() {
                write_section(
                    f,
                    "Return Values",
                    ret_values.iter(),
                    FormatType::List { literal: false },
                )?;
            }
        }

        // Error codes
        if let Some(err_codes) = &self.err_codes {
            if !err_codes.is_empty() {
                write_section(
                    f,
                    "Error Codes",
                    err_codes.iter(),
                    FormatType::List { literal: true },
                )?;
            }
        }

        // Implemenmtation guide lines
        if let Some(cop) = &self.cop {
            writeln!(f, "///")?;
            writeln!(f, "/// ### Conditions of Use")?;
            if !cop.before_call.is_empty() {
                writeln!(f, "/// #### Before Call")?;
                for line in &cop.before_call {
                    writeln!(f, "/// {}", remove_new_lines(line))?;
                }
            }
            if !cop.after_call.is_empty() {
                writeln!(f, "/// #### After Call")?;
                for line in &cop.after_call {
                    writeln!(f, "/// {}", remove_new_lines(line))?;
                }
            }
            if !cop.best_practice.is_empty() {
                writeln!(f, "/// ### Best Practice")?;
                for line in &cop.best_practice {
                    writeln!(f, "/// {}", remove_new_lines(line))?;
                }
            }
        }

        // See also ...
        if let Some(see_also) = &self.see_also {
            if !see_also.is_empty() {
                writeln!(f, "///")?;
                writeln!(f, "/// ### See Also")?;
                for reference in see_also {
                    writeln!(f, "/// * {}", reference.display)?;
                }
            }
        }

        // Usage
        if let Some(usage) = &self.usage {
            if !usage.is_empty() {
                write_section(f, "Usage", usage.iter(), FormatType::Code)?;
            }
        }

        Ok(())
    }
}

/// Generates formatted documentation comments for a given API.
///
/// This function takes the name of an API, constructs a file path by appending the API name to a
/// predefined source path, and then reads and transforms the corresponding JSON file at that path.
/// The transformed JSON is then parsed into an `ApiDescription` struct, which is used to generate
/// and return a string containing the formatted documentation comments for the API.
///
/// The transformation of the JSON file involves modifying certain structures within it, such as
/// `appliesTo` and `errCodes`, to fit a specific format expected by the `ApiDescription` parsing logic.
///
/// # Parameters
///
/// - `api`: A string slice that holds the relative path to the API JSON source file.
///
/// # Returns
///
/// Returns a `String` containing the formatted documentation comments for the specified API. If the JSON
/// file cannot be transformed, read, or parsed successfully, it returns a string indicating the failure.
///
/// # Examples
///
/// ```
/// let comments = generate_comments("my_api");
/// println!("{}", comments);
/// ```
///
/// Note: This example assumes that there is a JSON file named "my_api.json" in the predefined source
/// directory and that the `transform_input` function and `ApiDescription` struct are properly defined
/// and able to handle the JSON format of this file.
///
/// # Errors
///
/// This function will return an error message as a string if:
///
/// - The JSON file specified by the constructed file path cannot be opened or read.
/// - The contents of the JSON file cannot be successfully transformed or parsed into the `ApiDescription` struct.

pub fn generate_comments(file_path: &str) -> String {
    let json_string = transform_input::transform_input(file_path);

    // Read and parse the transformed JSON string into the ApiDescription struct
    match ApiDescription::from_modified_string(&json_string) {
        Ok(api_description) => api_description.to_string(),
        Err(e) => format!("Failed to parse JSON: {}", e),
    }
}
