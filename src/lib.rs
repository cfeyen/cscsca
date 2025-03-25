use phones::Phone;
use runtime::Runtime;
use runtime_cmds::PrintLog;

pub mod tokens;
pub mod phones;
pub mod meta_tokens;
pub mod rules;
pub mod applier;
pub mod runtime_cmds;

pub mod runtime;

#[cfg(test)]
mod tests;

pub const BOUND_STR: &str = "#";

/// Applies sca source code to an input string
/// 
/// Returns a string of either the final text or a formatted error and the print log
pub fn apply(input: &str, code: &str) -> (String, PrintLog) {
    let (result, log) = apply_fallible(input, code);

    (result.unwrap_or_else(|e| e), log)
}

/// Applies sca source code to an input string, logging prints
/// 
/// Returns a result of either the final text or a formatted error
#[inline]
pub fn apply_fallible(input: &str, code: &str) -> (Result<String, String>, PrintLog) {
    Runtime::default().apply(input, code)
}

/// Builds a list of phones (as string slices with lifetime 's)
/// from an input (string slice with 's)
/// and reformats whitespace as word bounderies
fn build_phone_list(input: &str) -> Vec<Phone<'_>> {
    let phones = input
        .split("")
        .filter(|s| !s.is_empty())
        .map(|s| if s == "\n" {
            s
        } else if s.trim().is_empty() {
            BOUND_STR
        } else {
            s
        })
        .map(Phone::new);

    let mut phone_list = Vec::new();

    for phone in phones {
        if phone.symbol() == "\n" {
            phone_list.push(Phone::new_bound());
            phone_list.push(phone);
            phone_list.push(Phone::new_bound());
        } else {
            phone_list.push(phone);
        }
    }

    phone_list
}

/// Converts a list of string slices to a string
/// reformating word bounderies as whitespace
fn phone_list_to_string(phone_list: &[Phone]) -> String {
    phone_list
        .iter()
        .fold(String::new(), |acc, phone| format!("{acc}{phone}"))
        .replace(&format!("{BOUND_STR}\n{BOUND_STR}"), "\n")
        .replace(BOUND_STR, " ")
        .trim()
        .to_string()
}

#[cfg(not(feature = "no_color"))]
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[93m";
    pub const GREEN: &str = "\x1b[92m";
    pub const BLUE: &str = "\x1b[94m";
    pub const MAGENTA: &str = "\x1b[35m";
}

#[cfg(feature = "no_color")]
pub mod colors {
    pub const RESET: &str = "";
    pub const BOLD: &str = "";
    pub const RED: &str = "";
    pub const YELLOW: &str = "";
    pub const GREEN: &str = "";
    pub const BLUE: &str = "";
    pub const MAGENTA: &str = "";
}

use colors::*;

/// Formats an error with its enviroment
fn format_error(e: &dyn std::error::Error, line: &str, line_num: usize) -> String {
    format!("{RED}Error:{RESET} {e}\nLine {line_num}: {line}")
}

/// prints the characters in a string
pub fn print_chars(text: &str) {
    println!("Characters in '{BLUE}{text}{RESET}':");

    for (i, c) in text.chars().enumerate().map(|(i, c)| (i + 1, c)) {
        println!("{i}:\t{c} ~ '{YELLOW}{}{RESET}'", c.escape_default());
    }
}

/// color formats then prints the help file
pub fn help() {
    let text = &mut include_str!("assets/help.txt").chars();
    let mut help = String::new();

    while let Some(c) = text.next() {
        match c {
            '[' => {
                let mut content = String::new();

                // gets bracket contents
                for c in text.by_ref() {
                    if c == ']' { break; }
                    content.push(c)
                }

                let special = match content.as_str() {
                    "-" => RESET,
                    "r" => { help += BOLD; RED },
                    "y" => YELLOW,
                    "g" => GREEN,
                    "b" => BLUE,
                    "m" => { help += BOLD; MAGENTA },
                    "!" => BOLD,
                    content => { help = help + "[" + content; "]" },
                };

                help += special;
            }
            c => help.push(c)
        }
    }

    println!("{help}");
}

/// returns the demo file
pub const fn demo() -> &'static str {
    include_str!("assets/demo.sca")
}

/// returns the template file
pub const fn template() -> &'static str {
    include_str!("assets/base.sca")
}