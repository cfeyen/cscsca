use std::collections::HashMap;

use phones::Phone;
use rules::RuleLine;
use runtime_cmd::{PrintLogs, RuntimeCmd};
use tokens::{ir::IrToken, token_checker};

pub mod tokens;
pub mod phones;
pub mod meta_tokens;
pub mod rules;
pub mod applier;
pub mod runtime_cmd;

#[cfg(feature = "async_apply")]
pub mod async_cscsca;

#[cfg(test)]
mod tests;

pub const BOUND_STR: &str = "#";

/// Applies sca source code to an input string
/// 
/// Returns a string of either the final text or a formatted error and the print logs
pub fn apply(input: &str, code: &str) -> (String, PrintLogs) {
    let mut logs = PrintLogs::default();
    let result = apply_fallible(input, code, &mut logs);

    (result.unwrap_or_else(|e| e), logs)
}

/// Applies sca source code to an input string, logging prints
/// 
/// Returns a result of either the final text or a formatted error
pub fn apply_fallible(input: &str, code: &str, print_logs: &mut PrintLogs) -> Result<String, String> {
    let mut definitions = HashMap::new();
    let lines_with_nums = code_by_line(code);
    let mut phone_list = build_phone_list(input);

    for (line_num, line) in lines_with_nums {
        let rule_line = build_rule(line, line_num, &mut definitions)?;

        match rule_line {
            RuleLine::Rule(rule) => {
                applier::apply(&rule, &mut phone_list)
                    .map_err(|e| format_error(e, line, line_num))?
            }
            RuleLine::Empty => (),
            RuleLine::Cmd(cmd, args) => handle_runtime_cmd(cmd, args, &phone_list, print_logs),
        }
    }

    Ok(phone_list_to_string(&phone_list))
}

/// Executes runtime commends
fn handle_runtime_cmd(cmd: RuntimeCmd, args: &str, phone_list: &[Phone], logs: &mut PrintLogs) {
    match cmd {
        RuntimeCmd::Print => {
            let output = format!("{args} '{BLUE}{}{RESET}'", phone_list_to_string(phone_list));
            #[cfg(not(feature = "no_runtime_print"))]
            println!("{output}");
            logs.log(output);
        }
    }
}

/// Converts code to an iterator of each line with the line number attached
fn code_by_line(code: &str) -> impl Iterator<Item = (usize, &str)> {
    code
        .lines()
        .enumerate()
        .map(|(num, line)| (num + 1, line))
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

/// Converts a line to a rule
/// 
/// Returns any errors as a formated string
fn build_rule<'s>(line: &'s str, line_num: usize, definitions: &mut HashMap<&'s str, Vec<IrToken<'s>>>) -> Result<RuleLine<'s>, String> {
    let tokens = tokens::tokenize_line_or_create_runtime_command(line, definitions)
        .map_err(|e| format_error(e, line, line_num))?;

    token_checker::check_token_line(&tokens)
        .map_err(|e| format_error(e, line, line_num))?;

    rules::build_rule(&tokens)
        .map_err(|e| format_error(e, line, line_num))
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
fn format_error(e: impl std::error::Error, line: &str, line_num: usize) -> String {
    format!("{RED}Error:{RESET} {e}\nLine {line_num}: {line}")
}

/// prints the characters in a string
pub fn print_chars(text: &str) {
    println!("Characters in '{BLUE}{text}{RESET}':");

    for (i, c) in text.chars().enumerate().map(|(i, c)| (i + 1, c)) {
        println!("{i}:\t{c} ~ '{YELLOW}{}{RESET}'", format!("{c:?}").replace("'", ""));
    }
}

/// color formats then prints the help file
pub fn help() {
    let text = &mut include_str!("help.txt").chars();
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
pub fn demo() -> &'static str {
    include_str!("demo.sca")
}

/// returns the template file
pub fn template() -> &'static str {
    include_str!("base.sca")
}