use std::{env, fs};

mod color;
use color::*;

const APPLY_CMD: &str = "sca";
const APPLY_TO_FILE_CMD: &str = "apply";
#[cfg(feature = "gen_vscode_grammar")]
const GEN_CMD: &str = "gen";
#[cfg(feature = "gen_vscode_grammar")]
const VSC_EXT: &str = "vscode_grammar";
const CHAR_HELP_CMD: &str = "chars";
const HELP_CMD: &str = "help";
const DEMO_CMD: &str = "demo";
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
    let args = &mut env::args();
    let _path = args.next();
    let cmd = args.next();
    let args = args.collect::<Vec<_>>();

    if let Some(cmd) = cmd {
        match (cmd.as_str(), args.len()) {
            (APPLY_CMD, 2..) => {
                let path = &args[0];
                let text = &args[1..].join(" ");

                let code = &match fs::read_to_string(path) {
                    Ok(code) => code,
                    Err(_) => {
                        println!("{RED}Error:{RESET} Could not find file '{BLUE}{path}{RESET}'");
                        return;
                    }
                };

                println!("{GREEN}Applying changes to '{BLUE}{text}{GREEN}'{RESET}");

                let output = cscsca::apply(text, code);
                println!("{}", output);
            },
            // prints the result of appling code from one file to text in another
            (APPLY_TO_FILE_CMD, 2) => {
                let src = &args[0];
                let target = &args[1];

                let code = &match fs::read_to_string(src) {
                    Ok(code) => code,
                    Err(_) => {
                        println!("{RED}Error:{RESET} Could not find file '{BLUE}{src}{RESET}'");
                        return;
                    }
                };

                let text = &match fs::read_to_string(target) {
                    Ok(code) => code,
                    Err(_) => {
                        println!("{RED}Error:{RESET} Could not find file '{BLUE}{target}{RESET}'");
                        return;
                    }
                };

                println!("{GREEN}Applying changes to '{BLUE}{text}{GREEN}'{RESET}");

                let output = cscsca::apply(text, code);
                println!("{}", output);
            },
            // stores the result of appling code from one file to text in another in a third
            // (either by overwriting its contents or creating a new one)
            (APPLY_TO_FILE_CMD, 3) => {
                let src = &args[0];
                let target = &args[1];
                let dest = &args[2];

                let code = &match fs::read_to_string(src) {
                    Ok(code) => code,
                    Err(_) => {
                        println!("{RED}Error:{RESET} Could not find file '{BLUE}{src}{RESET}'");
                        return;
                    }
                };

                let text = &match fs::read_to_string(target) {
                    Ok(code) => code,
                    Err(_) => {
                        println!("{RED}Error:{RESET} Could not find file '{BLUE}{target}{RESET}'");
                        return;
                    }
                };

                println!("{GREEN}Applying changes to '{BLUE}{text}{GREEN}'{RESET}");

                let output = cscsca::apply(text, code);
                println!("{}", output);

                match fs::write(dest, output) {
                    Ok(()) => println!("Done"),
                    Err(_) => println!("Could not create file '{dest}'")
                }
            },
            #[cfg(feature = "gen_vscode_grammar")]
            (GEN_CMD, 2) => {
                match &*args[0] {
                    VSC_EXT => {
                        let path = &*args[1];
                        if let Err(e) = cscsca::tooling_gen::vscode_grammar::gen_vscode_grammar(path) {
                            println!("{e}");
                        }
                    },
                    arg => {
                        println!("Unrecognized argument to {BOLD}{GEN_CMD}{RESET} '{arg}'");
                        println!("Run '{BOLD}cscsca help{RESET}' for more information");
                    },
                }
            }
            // prints the characters in each argument
            (CHAR_HELP_CMD, 1..) => {
                for text in &args {
                    print_chars(text);
                }
            },
            // prints the help file
            (HELP_CMD, 0) => help(),
            // creats a new template file
            (NEW_CMD, 1) => {
                let path = if args[0].contains(".") {
                    &args[0]
                } else {
                    &format!("{}{FILE_EXTENTION}", args[0])
                };
                
                match fs::write(path, template()) {
                    Ok(()) => println!("Created {BLUE}{path}{RESET}"),
                    Err(_) => println!("{RED}Error:{RESET} Failed to create {BLUE}{path}{RESET}")
                }
            },
            // prints the demo file
            (DEMO_CMD, 0) => println!("{}", demo()),
            // handles unrecognized commands
            (_, arg_count) => {
                println!("Unrecognized command '{BOLD}{cmd}{RESET}' with {arg_count} arguments");
                println!("Run '{BOLD}cscsca help{RESET}' for more information");
            }
        }
    } else {
        help()
    }
}

/// prints the characters in a string
fn print_chars(text: &str) {
    println!("Characters in '{BLUE}{text}{RESET}':");

    for (i, c) in text.chars().enumerate().map(|(i, c)| (i + 1, c)) {
        println!("{i}:\t{c} ~ '{YELLOW}{}{RESET}'", c.escape_default());
    }
}

/// color formats then prints the help file
fn help() {
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
const fn demo() -> &'static str {
    include_str!("assets/demo.sca")
}

/// returns the template file
const fn template() -> &'static str {
    include_str!("assets/base.sca")
}