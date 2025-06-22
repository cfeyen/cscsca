#![allow(clippy::non_minimal_cfg)]

const USE_TEMPLATE_FLAGS: &[&str] = &["-t", "--template"];
const CHAIN_FLAGS: &[&str] = &["-c", "--chain"];
const READ_FLAGS: &[&str] = &["-r", "--read"];
const WRITE_FLAGS: &[&str] = &["-w", "--write"];
const MAPPED_OUTPUT_FLAGS: &[&str] = &["-m", "--map"];
const MAP_SEPARATOR_FLAGS: &[&str] = &["-s", "--separator"];

const DEFAULT_MAP_SPACER: &str = "->";

use std::env;

use crate::{APPLY_CMD, CHAR_HELP_CMD, HELP_CMD, NEW_CMD};
#[cfg(any(feature = "gen_vscode_grammar"))]
use crate::GEN_CMD;
#[cfg(feature = "gen_vscode_grammar")]
use crate::VSC_EXT;

/// Parsed CLI input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    Apply {
        paths: Vec<String>,
        output_data: OutputData,
        input: InputType,
    },
    Chars { words: Vec<String> },
    Help { extra_args: bool },
    New {
        use_template: bool,
        path: String,
    },
    #[cfg(any(feature = "gen_vscode_grammar"))]
    Gen {
        tooling: GenType,
        path: String,
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
    Raw(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputData {
    write: Option<String>,
    map: Option<String>,
}

impl OutputData {
    pub const fn map(&self) -> Option<&String> {
        self.map.as_ref()
    }

    pub const fn write_path(&self) -> Option<&String> {
        self.write.as_ref()
    }
}

impl CliCommand {
    /// Parses the CLI arguments into a command that can be executed by `main`
    pub fn from_args() -> Result<Self, ArgumentParseError> {
        let mut args = env::args().peekable();
        let _path = args.next();

        match args.next() {
            Some(cmd) => match cmd.as_str() {
                APPLY_CMD => parse_sca(&mut args),
                CHAR_HELP_CMD => Ok(Self::Chars { words: args.collect() }),
                NEW_CMD => {
                    let use_template = args.next_if(|s| USE_TEMPLATE_FLAGS.contains(&s.as_str())).is_some();

                    let Some(path) = args.next() else {
                        return Err(ArgumentParseError::ExpectedFileName);
                    };
                    
                    if let Some(cmd) = args.next() {
                        return Err(ArgumentParseError::UnexpectedCommand(cmd));
                    }

                    Ok(Self::New { use_template, path })
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

                    if let Some(cmd) = args.next() {
                        return Err(ArgumentParseError::UnexpectedCommand(cmd));
                    }

                    Ok(CliCommand::Gen { tooling, path })
                },
                HELP_CMD => Ok(Self::Help { extra_args: args.next().is_some() }),
                _ => Err(ArgumentParseError::UnexpectedCommand(cmd)),
            }
            None => Ok(CliCommand::None),
        }
    }
}

/// Parses the application command's arguments
fn parse_sca(args: &mut std::iter::Peekable<env::Args>) -> Result<CliCommand, ArgumentParseError> {
    let mut paths = Vec::new();

    loop {
        if let Some(path) = args.next() {
            paths.push(path);
        } else {
            return Err(ArgumentParseError::ExpectedFileName);
        }

        if args.next_if(|s| CHAIN_FLAGS.contains(&s.as_str())).is_none() {
            break;
        }
    }
    
    let map = if args.next_if(|s| MAPPED_OUTPUT_FLAGS.contains(&s.as_str())).is_some() {
        if args.next_if(|s| MAP_SEPARATOR_FLAGS.contains(&s.as_str())).is_some() {
            if let Some(sep) = args.next() {
                Some(sep)
            } else {
                return Err(ArgumentParseError::ExpectedSeparator);
            }
        } else {
            Some(DEFAULT_MAP_SPACER.to_string())
        }
    } else {
        None
    };

    let write = if args.next_if(|s| WRITE_FLAGS.contains(&s.as_str())).is_some() {
        match args.next() {
            Some(path) => Some(path),
            None => return Err(ArgumentParseError::ExpectedFileName),
        }
    } else {
        None
    };

    let input = if args.next_if(|s| READ_FLAGS.contains(&s.as_str())).is_some() {
        match args.next() {
            Some(path) => InputType::Read(path),
            None => return Err(ArgumentParseError::ExpectedFileName),
        }
    } else {
        InputType::Raw(args.collect::<Vec<_>>().join(" "))
    };

    if let Some(cmd) = args.next() {
        return Err(ArgumentParseError::UnexpectedCommand(cmd));
    }

    Ok(CliCommand::Apply { paths, output_data: OutputData { write, map }, input })
}

#[derive(Debug)]
pub enum ArgumentParseError {
    UnexpectedCommand(String),
    ExpectedFileName,
    ExpectedSeparator,
    #[cfg(any(feature = "gen_vscode_grammar"))]
    ExpectedCommand,
}

impl std::error::Error for ArgumentParseError {}

impl std::fmt::Display for ArgumentParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedCommand(cmd) => write!(f, "Unexpected command '{cmd}'"),
            Self::ExpectedFileName => write!(f, "Input ended unexpectedly, expected a file name"),
            Self::ExpectedSeparator => write!(f, "Expected a seperator after flag {}", MAP_SEPARATOR_FLAGS.join(" or ")),
            #[cfg(any(feature = "gen_vscode_grammar"))]
            Self::ExpectedCommand => write!(f, "Input ended unexpectedly, expected a command"),
        }
    }
}