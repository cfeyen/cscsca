//! # CSCSCA
//! CSCSCA (Charles' Super Cool Sound Change Applier) is a tool for simulating phonentic sound change,
//! applying written rules (see `README.md`) to an input.

use std::{fs, fmt::Write as _};

mod color;
mod cli_parser;

use cli_parser::{CliCommand, InputType, OutputData};
use color::{BLUE, BOLD, GREEN, RED, RESET, YELLOW};

const APPLY_CMD: &str = "sca";
const CHAR_HELP_CMD: &str = "chars";
const HELP_CMD: &str = "help";
const NEW_CMD: &str = "new";
const FILE_EXTENTION: &str = ".sca";

/// Reads the command line arguments and acts upon them
/// 
/// See `README.md` for more information
fn main() {
    match CliCommand::from_args() {
        Ok(CliCommand::Apply { paths, output_data, input })
            => run_apply(&paths, &output_data, input),
        Ok(CliCommand::Chars { words }) => for text in words {
            print_chars(&text);
        },
        Ok(CliCommand::New { use_template, path }) => {
            let path = path + FILE_EXTENTION;

            if std::path::Path::new(&path).exists() {
                error(&format!("{BLUE}{path}{RESET} already exisits"));
            } else if fs::write(&path, if use_template { template() } else { "" }).is_err() {
                error(&format!("An error occured when writing to {BLUE}{path}{RESET}"));
            }
        },
        Ok(CliCommand::Help { extra_args }) => {
            if extra_args {
                warn(&format!("arguments beyond '{BOLD}{HELP_CMD}{RESET}' do nothing"));
            }
            help();
        },
        Ok(CliCommand::None) => {
            println!("Charles' Super Cool Sound Change Applier");
            println!("Run '{BOLD}cscsca help{RESET}' for more information");
        },
        Err(e) => error(&e.to_string()),
    }
}

/// Applies changes to every input from CLI data
fn run_apply(paths: &[String], output_data: &OutputData, input_type: InputType) {
    let input = match input_type {
        InputType::Raw(raw) if raw.is_empty() => return error("No input provided"),
        InputType::Raw(raw) => raw,
        InputType::Read(path) if path.is_empty() => return error("No input provided"),
        InputType::Read(path) => match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => return error(&format!("An error occured when reading the input from {BLUE}{path}{RESET}")),
        },
    };

    let mut full_output = String::new();
    let build = input.contains('\n');

    if build {
        let Ok(rule_sets) = paths.iter()
            .map(fs::read_to_string)
            .collect::<Result<Vec<_>, _>>() else {
                return error("Could not find file '{BLUE}{path}{RESET}'");
            };
        
        let appliable_rule_sets = match rule_sets.iter()
            .map(|rule_set| cscsca::build_rules(rule_set, &mut cscsca::CliGetter))
            .collect::<Result<Vec<_>, _>>() {
                Ok(rules) => rules,
                Err(e) => return println!("{e}"),
            };
        
        let mut runtime = cscsca::CliRuntime::default();

        for input in input.lines() {
            let line_output = apply_rule_sets(paths, output_data, &appliable_rule_sets, &mut runtime, input);
            _ = writeln!(full_output, "{line_output}");
        }
    } else {
        for input in input.lines() {
            match apply_changes(paths, input.to_string(), output_data.map()) {
                Ok(output) => {
                    full_output += &output;
                    println!("{output}");
                },
                Err(e) => return println!("{e}"),
            }

            full_output.push('\n');
        }
    }

    if let Some(path) = output_data.write_path() {
        if fs::write(path, full_output).is_err() {
            error(&format!("An error occured when writing the output to {BLUE}{path}{RESET}"));
        }
    }
}

fn apply_rule_sets(paths: &[String], output_data: &OutputData, appliable_rule_sets: &[cscsca::AppliableRules<'_>], runtime: &mut cscsca::CliRuntime, input: &str) -> String {
    let mut line_output = if output_data.map().is_some() {
        input.to_string()
    } else {
        String::new()
    };
            
    let mut input = input.to_string();

    for (i, rule_set) in appliable_rule_sets.iter().enumerate() {
        println!("{GREEN}Applying changes in {BLUE}{}{GREEN} to '{BLUE}{input}{GREEN}'{RESET}", &paths[i]);

        match rule_set.apply_fallible(&input, runtime) {
            Ok(output) => {
                if let Some(sep) = output_data.map() {
                    _ = write!(line_output, " {sep} {output}");
                } else {
                    _ = write!(line_output, "{output}");
                }
                input = output;
            },
            Err(e) => {
                println!("{e}");
                break;
            },
        }
    }

    println!("{line_output}");
    line_output
}

/// Applies changes to an input
fn apply_changes(paths: &[String], mut input: String, map: Option<&String>) -> Result<String, String> {
    let mut full_output = if map.is_some() {
        input.clone()
    } else {
        String::new()
    };

    for path in paths {
        let code = &match fs::read_to_string(path) {
            Ok(code) => code,
            Err(_) => {
                return Err(format!("Could not find file '{BLUE}{path}{RESET}'"));
            }
        };

        println!("{GREEN}Applying changes in {BLUE}{path}{GREEN} to '{BLUE}{input}{GREEN}'{RESET}");

        match cscsca::apply_fallible(&input, code) {
            Ok(output) => {
                if let Some(sep) = map {
                    _ = write!(full_output, " {sep} {output}");
                }
                input = output;
            },
            Err(e) => return Err(e.to_string()),
        }
    }

    if map.is_none() {
        full_output = input;
    }

    Ok(full_output)
}

/// prints the characters in a string
fn print_chars(text: &str) {
    println!("Characters in '{BLUE}{text}{RESET}':");

    for (i, c) in text.chars().enumerate().map(|(i, c)| (i + 1, c)) {
        println!("{i}:\t{c} ~ '{YELLOW}{}{RESET}'", c.escape_default());
    }
}

/// Prints an error
fn error(e: &str) {
    println!("{RED}Error:{RESET} {e}");
}

/// Prints a warning
fn warn(w: &str) {
    println!("{YELLOW}Warning:{RESET} {w}");
}

/// prints the README fule
fn help() {
    println!("{}", include_str!("../README.md"));
}

/// returns the template file
const fn template() -> &'static str {
    include_str!("assets/template.sca")
}