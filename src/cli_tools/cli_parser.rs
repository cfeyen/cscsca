const USE_TEMPLATE_FLAGS: [&str; 2] = ["-t", "--template"];
const CHAIN_FLAGS: [&str; 2] = ["-c", "--chain"];
const READ_FLAGS: [&str; 2] = ["-r", "--read"];
const WRITE_FLAGS: [&str; 2] = ["-w", "--write"];
const QUIET_FLAGS: [&str; 2] = ["-q", "--quiet"];
const MAP_OUTPUT_FLAGS: [&str; 2] = ["-o", "--map_outputs"];
const MAP_LOGS_FLAGS: [&str; 2] = ["-p", "--map_prints"];
const MAP_ALL_FLAGS: [&str; 2] = ["-m", "--map_all"];
const REDUCE_OUTPUT_FLAGS: [&str; 2] = ["-x", "--reduce"];
const MAP_SEPARATOR_FLAGS: [&str; 2] = ["-s", "--separator"];

const DEFAULT_MAP_SPACER: &str = "->";

use std::env;

use crate::{cli_tools::ansi::{BOLD, RESET}, APPLY_CMD, CHAR_HELP_CMD, HELP_CMD, NEW_CMD};

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
    None,
}

/// How input should be read
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputType {
    /// Read from file
    Read(String),
    /// Read from the end of cli input
    Raw(String),
}

/// How output is displayed
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputData {
    /// What file the output should be written to
    write: Option<String>,
    /// How intermediate stages should be displayed
    map: Option<MapData>,
    /// If intermediate stages should be printed during runtime
    quiet: bool,
}

impl OutputData {
    /// Gets the map data
    pub const fn map_data(&self) -> Option<&MapData> {
        self.map.as_ref()
    }

    /// Gets the file the output should be written to
    pub const fn write_path(&self) -> Option<&String> {
        self.write.as_ref()
    }

    /// Gets if the quiet flag is set
    pub const fn quiet(&self) -> bool {
        self.quiet
    }
}

/// Which outputs are displayed in the final output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapType {
    /// Only final phonological form should be output
    Final,
    /// Only print statements should be output
    Logs,
    /// Both print statements and final phonological form should be output
    FinalAndLogs,
}

/// How intermediate outputs should be displayed
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapData {
    /// Which outputs should be displayed in the final output
    map_type: MapType,
    /// If consecutive duplicate outputs should be removed
    reduce: bool,
    /// What symbol should seperate outputs
    sep: String,
}

impl MapData {
    /// Gets the map type
    pub fn map_type(&self) -> MapType {
        self.map_type
    }

    /// Whether or not the output should be deduped
    pub fn reduce(&self) -> bool {
        self.reduce
    }

    /// Returns the map separator
    pub fn sep(&self) -> &str {
        &self.sep
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

    // adds each file path to `paths`
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
    
    // sets the mapping type
    let map_type = if args.next_if(|s| MAP_OUTPUT_FLAGS.contains(&s.as_str())).is_some() {
        Some(MapType::Final)
    } else if args.next_if(|s| MAP_ALL_FLAGS.contains(&s.as_str())).is_some() {
        Some(MapType::FinalAndLogs)
    } else if args.next_if(|s| MAP_LOGS_FLAGS.contains(&s.as_str())).is_some() {
        Some(MapType::Logs)
    } else {
        None
    };

    // builds the mapping data
    let map = if let Some(map_type) = map_type {
        let reduce = args.next_if(|s| REDUCE_OUTPUT_FLAGS.contains(&s.as_str())).is_some();

        let sep = if args.next_if(|s| MAP_SEPARATOR_FLAGS.contains(&s.as_str())).is_some() {
            if let Some(sep) = args.next() {
                sep
            } else {
                return Err(ArgumentParseError::ExpectedSeparator);
            }
        } else {
            DEFAULT_MAP_SPACER.to_string()
        };

        Some(MapData {
            map_type,
            reduce,
            sep,
        })
    } else {
        None
    };

    // sets the quite flag
    let quiet = args.next_if(|s| QUIET_FLAGS.contains(&s.as_str())).is_some();

    // sets the write files
    let write = if args.next_if(|s| WRITE_FLAGS.contains(&s.as_str())).is_some() {
        match args.next() {
            Some(path) => Some(path),
            None => return Err(ArgumentParseError::ExpectedFileName),
        }
    } else {
        None
    };

    // sets the input type
    let input = if args.next_if(|s| READ_FLAGS.contains(&s.as_str())).is_some() {
        match args.next() {
            Some(path) => InputType::Read(path),
            None => return Err(ArgumentParseError::ExpectedFileName),
        }
    } else {
        InputType::Raw(args.collect::<Vec<_>>().join(" "))
    };

    // returns an error for extra arguments
    if let Some(cmd) = args.next() {
        return Err(ArgumentParseError::UnexpectedCommand(cmd));
    }

    // constructs the apply command
    Ok(CliCommand::Apply { paths, output_data: OutputData { write, map, quiet }, input })
}

/// An error caused by invalid cli input
#[derive(Debug)]
pub enum ArgumentParseError {
    UnexpectedCommand(String),
    ExpectedFileName,
    ExpectedSeparator,
}

impl std::error::Error for ArgumentParseError {}

impl std::fmt::Display for ArgumentParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedCommand(cmd) => writeln!(f, "Unexpected command '{cmd}'")?,
            Self::ExpectedFileName => writeln!(f, "Input ended unexpectedly, expected a file name")?,
            Self::ExpectedSeparator => writeln!(f, "Expected a seperator after flag {}", MAP_SEPARATOR_FLAGS.join(" or "))?,
        }

        write!(f, "Run '{BOLD}cscsca help{RESET}' for more information")
    }
}