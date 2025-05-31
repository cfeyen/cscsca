use std::env::Args;

use crate::{APPLY_CMD, CHAR_HELP_CMD, HELP_CMD, NEW_CMD};
#[cfg(any(feature = "gen_vscode_grammar"))]
use crate::GEN_CMD;
#[cfg(feature = "gen_vscode_grammar")]
use crate::VSC_EXT;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    Apply {
        write: Option<String>,
        paths: Vec<String>,
        input: InputType
    },
    Chars { words: Vec<String> },
    Help { extra_args: bool },
    New {
        base: bool,
        path: String,
        extra_args: bool
    },
    #[cfg(any(feature = "gen_vscode_grammar"))]
    Gen {
        tooling: GenType,
        path: String,
        extra_args: bool
    },
    None,
}

#[cfg(any(feature = "gen_vscode_grammar"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenType {
    #[cfg(feature = "gen_vscode_grammar")]
    VsCodeGrammar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputType {
    Read(String),
    Raw(String)
}

impl CliCommand {
    pub fn from_args(args: Args) -> Result<Self, ArgumentParseError> {
        let mut args = args.peekable();
        let _path = args.next();

        match args.next() {
            Some(cmd) => match cmd.as_str() {
                APPLY_CMD => parse_sca(&mut args),
                CHAR_HELP_CMD => Ok(Self::Chars { words: args.collect() }),
                NEW_CMD => {
                    let base = args.next_if(|s| matches!(s.as_str(), "--base" | "-b")).is_some();

                    let path = match args.next() {
                        Some(path) => path,
                        None => return Err(ArgumentParseError::ExpectedFileName),
                    };
                    let extra_args = args.next().is_some();

                    Ok(Self::New { base, path, extra_args })
                }
                #[cfg(any(feature = "gen_vscode_grammar"))]
                GEN_CMD => {
                    let tooling = if let Some(cmd) = args.next() {
                        match cmd.as_str() {
                            VSC_EXT => GenType::VsCodeGrammar,
                            _ => return Err(ArgumentParseError::UnexpectedCommand(cmd)),
                        }
                    } else {
                        return Err(ArgumentParseError::ExpectedCommand);
                    };

                    let path = match args.next() {
                        Some(path) => path,
                        None => return Err(ArgumentParseError::ExpectedFileName),
                    };

                    Ok(CliCommand::Gen { tooling, path, extra_args: args.next().is_some()})
                },
                HELP_CMD => Ok(Self::Help { extra_args: args.next().is_some() }),
                _ => Err(ArgumentParseError::UnexpectedCommand(cmd)),
            }
            None => Ok(CliCommand::None),
        }
    }
}

fn parse_sca(args: &mut std::iter::Peekable<impl Iterator<Item = String>>) -> Result<CliCommand, ArgumentParseError> {
    let mut paths = Vec::new();

    loop {
        if let Some(path) = args.next() {
            paths.push(path);
        } else {
            return Err(ArgumentParseError::ExpectedFileName);
        }

        if matches!(args.peek().map(|s| s.as_str()), Some("--chain" | "-c")) {
            _ = args.next();
        } else {
            break;
        }
    }

    let write = if args.next_if(|s| matches!(s.as_str(), "--write" | "-w")).is_some() {
        match args.next() {
            Some(path) => Some(path),
            None => return Err(ArgumentParseError::ExpectedFileName),
        }
    } else {
        None
    };

    let input = if args.next_if(|s| matches!(s.as_str(), "--read" | "-r")).is_some() {
        let input = match args.next() {
            Some(path) => InputType::Read(path),
            None => return Err(ArgumentParseError::ExpectedFileName),
        };

        if let Some(cmd) = args.next() {
            return Err(ArgumentParseError::UnexpectedCommand(cmd));
        } else {
            input
        }
    } else {
        InputType::Raw(args.collect::<Vec<_>>().join(" "))
    };

    Ok(CliCommand::Apply { write, paths, input })
}

#[derive(Debug)]
pub enum ArgumentParseError {
    UnexpectedCommand(String),
    ExpectedFileName,
    #[cfg(any(feature = "gen_vscode_grammar"))]
    ExpectedCommand,
}

impl std::error::Error for ArgumentParseError {}

impl std::fmt::Display for ArgumentParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedCommand(cmd) => write!(f, "Unexpected command '{cmd}'"),
            Self::ExpectedFileName => write!(f, "Input ended unexpectedly, expected a file name"),
            #[cfg(any(feature = "gen_vscode_grammar"))]
            Self::ExpectedCommand => write!(f, "Input ended unexpectedly, expected a command"),
        }
    }
}