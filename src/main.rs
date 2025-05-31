use std::{env, fs};

mod color;
mod cli_parser;

use cli_parser::{CliCommand, InputType};
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

/// Entry point
/// 
/// Reads the command line arguments and acts upon them
/// - help - formats and prints the help file
/// - demo - prints the demo file
/// - new 'name' - creates a template file with the name 'name'
/// - chars 'text'+ - prints each character in each 'text'+
/// - sca 'path' 'text'+ - applies the rules in the file at 'path' to 'text'+
/// - apply 'src' 'path' - applies the code at 'src' to the text in 'path' and prints the result
/// - apply 'src' 'path' 'dest' - applies the code at 'src' to the text in 'path' and stores the result in 'dest'
fn main() {
    match CliCommand::from_args(env::args()) {
        Ok(CliCommand::Apply { write, paths, input }) => {
            let text = match input {
                InputType::Raw(raw) => raw,
                InputType::Read(path) => match fs::read_to_string(&path) {
                    Ok(s) => s,
                    Err(_) => {
                        println!("{RED}Error:{RESET} An error occured when reading the input from {BLUE}{path}{RESET}");
                        return;
                    },
                },
            };

            match apply_changes(&paths, text) {
                Ok(output) => {
                    println!("{output}");
                    if let Some(path) = write {
                        if fs::write(&path, output).is_err() {
                            println!("{RED}Error:{RESET} An error occured when writing the output to {BLUE}{path}{RESET}");
                        }
                    }
                },
                Err(e) => println!("{RED}Error:{RESET} {e}")
            }
        },
        Ok(CliCommand::Chars { words }) => for text in words {
            print_chars(&text);
        },
        Ok(CliCommand::New { base, path, extra_args }) => {
            if extra_args {
                println!("{YELLOW}Warning:{RESET} Extra arguments beyond '{BLUE}{path}{RESET}' do nothing");
            }

            let path = path + FILE_EXTENTION;

            if std::path::Path::new(&path).exists() {
                println!("{RED}Error:{RESET} {BLUE}{path}{RESET} already exisits");
            } else if fs::write(&path, if base { template() } else { "" }).is_err() {
                println!("{RED}Error:{RESET} An error occured when writing to {BLUE}{path}{RESET}");
            }
        },
        #[cfg(any(feature = "gen_vscode_grammar"))]
        Ok(CliCommand::Gen { tooling, path, extra_args }) => {
            if extra_args {
                println!("{YELLOW}Warning:{RESET} Extra arguments beyond '{BOLD}{path}{RESET}' do nothing");
            }

            match tooling {
                #[cfg(feature = "gen_vscode_grammar")]
                GenType::VsCodeGrammar => if let Err(e) = cscsca::tooling_gen::vscode_grammar::gen_vscode_grammar(&path) {
                    println!("{e}");
                }
            }
        }
        Ok(CliCommand::Help { extra_args }) => {
            if extra_args {
                println!("{YELLOW}Warning:{RESET} Extra arguments beyond '{BOLD}{HELP_CMD}{RESET}' do nothing");
            }
            help();
        },
        Ok(CliCommand::None) => {
            println!("Charles' Super Cool Sound Change Applier");
            println!("Run '{BOLD}cscsca help{RESET}' for more information");
        },
        Err(e) => println!("{RED}Error:{RESET} {e}"),
    }
}

fn apply_changes(paths: &[String], input: String) -> Result<String, String> {
    if input.is_empty() {
        return Err("No input provided".to_string())
    }

    let mut output = input;

    for path in paths {
        let code = &match fs::read_to_string(path) {
            Ok(code) => code,
            Err(_) => {
                return Err(format!("Could not find file '{BLUE}{path}{RESET}'"));
            }
        };

        println!("{GREEN}Applying changes in {BLUE}{path}{GREEN} to '{BLUE}{output}{GREEN}'{RESET}");

        match cscsca::apply_fallible(&output, code) {
            Ok(text) => output = text,
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(output)
}

/// prints the characters in a string
fn print_chars(text: &str) {
    println!("Characters in '{BLUE}{text}{RESET}':");

    for (i, c) in text.chars().enumerate().map(|(i, c)| (i + 1, c)) {
        println!("{i}:\t{c} ~ '{YELLOW}{}{RESET}'", c.escape_default());
    }
}

/// prints the README fule
fn help() {
    println!("{}", include_str!("../README.md"))
}

/// returns the template file
const fn template() -> &'static str {
    include_str!("assets/base.sca")
}