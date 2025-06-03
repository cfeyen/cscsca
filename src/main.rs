use std::{env, fs};

mod color;
mod cli_parser;

use cli_parser::{CliCommand, InputType, OutputData};
#[cfg(any(feature = "gen_vscode_grammar"))]
use cli_parser::GenType;
use color::*;

const APPLY_CMD: &str = "sca";
#[cfg(any(feature = "gen_vscode_grammar"))]
const GEN_CMD: &str = "gen";
#[cfg(feature = "gen_vscode_grammar")]
const VSC_EXT: &str = "vscode_grammar";
const CHAR_HELP_CMD: &str = "chars";
const HELP_CMD: &str = "help";
const NEW_CMD: &str = "new";
const FILE_EXTENTION: &str = ".sca";

const MAP_SPACER: &str = "->";

/// Reads the command line arguments and acts upon them
/// 
/// See `README.md` for more information
fn main() {
    match CliCommand::from_args(env::args()) {
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
        #[cfg(any(feature = "gen_vscode_grammar"))]
        Ok(CliCommand::Gen { tooling, path }) => match tooling {
            #[cfg(feature = "gen_vscode_grammar")]
            GenType::VsCodeGrammar => if let Err(e) = cscsca::tooling_gen::vscode_grammar::gen_vscode_grammar(&path) {
                error(&e.to_string());
            },
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
fn run_apply(paths: &[String], output_data: &OutputData, input: InputType) {
    let input = match input {
        InputType::Raw(raw) => raw,
        InputType::Read(path) => match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => {
                error(&format!("An error occured when reading the input from {BLUE}{path}{RESET}"));
                return;
            },
        },
    };

    let mut full_output = String::new();

    for input in input.lines() {
        match apply_changes(paths, input.to_string(), output_data.map()) {
            Ok(output) => {
                full_output += &output;
                println!("{output}");
            },
            Err(e) => error(&e),
        }

        full_output.push('\n');
    }

    if let Some(path) = output_data.write_path() {
        if fs::write(path, full_output).is_err() {
            error(&format!("An error occured when writing the output to {BLUE}{path}{RESET}"));
        }
    }
}

/// Applies changes to an input
fn apply_changes(paths: &[String], mut input: String, map: bool) -> Result<String, String> {
    if input.is_empty() {
        return Err("No input provided".to_string())
    }

    let mut full_output = if map {
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
                if map {
                    use std::fmt::Write as _;
                    _ = write!(full_output, " {MAP_SPACER} {output}");
                }
                input = output;
            },
            Err(e) => return Err(e.to_string()),
        }
    }

    if !map {
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
    println!("{RED}Error:{RESET} {e}")
}

/// Prints a warning
fn warn(w: &str) {
    println!("{YELLOW}Warning:{RESET} {w}")
}

/// prints the README fule
fn help() {
    println!("{}", include_str!("../README.md"))
}

/// returns the template file
const fn template() -> &'static str {
    include_str!("assets/base.sca")
}