//! # CSCSCA
//! CSCSCA (Charles' Super Cool Sound Change Applier) is a tool for simulating phonentic sound change,
//! applying written rules (see `README.md`) to an input.

#[cfg(feature = "async_io")]
compile_error! { "binary cannot be compiled with the feature flag `async_io`" }

use std::{fs, fmt::Write as _};

mod cli_tools;

use cli_tools::{
    ansi::{BLUE, BOLD, GREEN, RED, RESET, YELLOW},
    cli_parser::{MapData, MapType, CliCommand, InputType, OutputData},
    cli_io::{CliGetter, LogAndPrintRuntime},
};

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
        Ok(CliCommand::Apply { paths, output_data, input }) => {
            if let Err(e) = run_apply(&paths, &output_data, input) {
                println!("{e}");
            }
        },
        Ok(CliCommand::Chars { words }) => for text in words {
            print_chars(&text);
        },
        Ok(CliCommand::New { use_template, path }) => {
            let path = path + FILE_EXTENTION;

            if std::path::Path::new(&path).exists() {
                println!("{RED}Error: {BLUE}{path}{RESET} already exisits");
            } else if fs::write(&path, if use_template { template() } else { "" }).is_err() {
                println!("{RED}Error: {RESET}An error occured when writing to {BLUE}{path}{RESET}");
            }
        },
        Ok(CliCommand::Help { extra_args }) => {
            if extra_args {
                println!("{YELLOW} Warning: {RESET}Arguments beyond '{BOLD}{HELP_CMD}{RESET}' do nothing");
            }
            help();
        },
        Ok(CliCommand::None) => {
            println!("Charles' Super Cool Sound Change Applier");
            println!("Run '{BOLD}cscsca help{RESET}' for more information");
        },
        Err(e) => println!("{RED}Error: {RESET}{e}"),
    }
}

/// An error in the cli wrapper around sound change application
#[derive(Debug)]
enum CliError {
    CouldNotWrite(String),
    NoFile(String),
    NoInput,
}

impl std::error::Error for CliError {}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{RED}Error: {RESET}")?;

        match self {
            Self::CouldNotWrite(path) => write!(f, "Could not write to file: '{BLUE}{path}{RESET}'"),
            Self::NoInput => write!(f, "No input phones or source provided"),
            Self::NoFile(path) => write!(f, "Could not find file: '{BLUE}{path}{RESET}'"),
        }
    }
}

/// Applies changes to every input from CLI data
fn run_apply(paths: &[String], output_data: &OutputData, input_type: InputType) -> Result<(), CliError> {
    // gets the initial input
    let input = match input_type {
        InputType::Raw(raw) if raw.is_empty() => return Err(CliError::NoInput),
        InputType::Raw(raw) => raw,
        InputType::Read(path) if path.is_empty() => return Err(CliError::NoInput),
        InputType::Read(path) => match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => return Err(CliError::NoFile(path)),
        },
    };

    let mut full_output = String::new();
    // determines if rules should be pre-built or line-by-line interpretation
    let build = input.contains('\n');

    // gets each rule set in the chain
    let rule_sets = paths.iter()
        .map(|path| fs::read_to_string(path).map_err(|_| CliError::NoFile(path.clone())))
        .collect::<Result<Vec<_>, _>>()?;

    if build {
        // build each rule set into an appliable form
        let appliable_rule_sets = match rule_sets.iter()
            .map(|rule_set| cscsca::build_rules(rule_set, &mut CliGetter))
            .collect::<Result<Vec<_>, _>>() {
                Ok(rules) => rules,
                Err(e) => {
                    print_error(&e);
                    return Ok(());
                },
            };

        // applies each rule set in the rules chain to each line of the input
        for input in input.lines() {
            let line_output = match apply_rule_sets(paths, output_data, &appliable_rule_sets, input.to_string()) {
                Ok(out) => {
                    println!("{out}");
                    out
                },
                Err(e) => {
                    print_error(&e);
                    format!("{e}")
                },
            };

            // records the output
            _ = writeln!(full_output, "{line_output}");
        }
    } else {
        // applies each rule set in the chain to the input
        match apply_changes(paths, output_data, &rule_sets, input) {
            Ok(output) => {
                // records the output
                full_output += &output;
                println!("{output}");
            },
            Err(e) => {
                print_error(&e);
                return Ok(());
            },
        }

        full_output.push('\n');
    }

    // writes output to the output file if it exists
    if let Some(path) = output_data.write_path()
        && fs::write(path, full_output).is_err()
    {
        Err(CliError::CouldNotWrite(path.to_string()))
    } else {
        Ok(())
    }
}

/// Applies each pre-built rule set to an input
fn apply_rule_sets(paths: &[String], output_data: &OutputData, rule_sets: &[cscsca::AppliableRules<'_>], input: String) -> Result<String, cscsca::ScaError> {
    let mut mapping = new_mapping(output_data.map_data().map(MapData::map_type), &input);

    let mut last_output = input;

    let mut runtime = if output_data.quiet() {
        AppRuntime::Quiet(cscsca::LogRuntime::default())
    } else {
        AppRuntime::Loud(LogAndPrintRuntime::default())
    };

    // applies each rule set
    for (i, rule_set) in rule_sets.iter().enumerate() {
        println!("{GREEN}Applying changes in {BLUE}{}{GREEN} to '{BLUE}{last_output}{GREEN}'{RESET}", &paths[i]);

        let set_output = rule_set.apply_fallible(&last_output, &mut runtime)?;
        
        if let Some(map_data) = output_data.map_data() {
            extend_mapping(map_data.map_type(), &set_output, &mut mapping, &mut runtime);
        }

        // records output
        last_output = set_output;
    }

    Ok(mapped_output(output_data.map_data(), mapping, last_output))
}

/// Applies each rule set to an input
fn apply_changes(paths: &[String], output_data: &OutputData, rule_sets: &[String], input: String) -> Result<String, cscsca::ScaError> {
    let mut mapping = new_mapping(output_data.map_data().map(MapData::map_type), &input);

    let mut last_output = input;

    let runtime = if output_data.quiet() {
        AppRuntime::Quiet(cscsca::LogRuntime::default())
    } else {
        AppRuntime::Loud(LogAndPrintRuntime::default())
    };

    let mut executor = cscsca::LineByLineExecuter::new(runtime, CliGetter);

    // applies each rule set
    for (i, rule_set) in rule_sets.iter().enumerate() {
        println!("{GREEN}Applying changes in {BLUE}{}{GREEN} to '{BLUE}{last_output}{GREEN}'{RESET}", &paths[i]);

        let set_output = executor.apply_fallible(&last_output, rule_set)?;

        if let Some(map_data) = output_data.map_data() {
            extend_mapping(map_data.map_type(), &set_output, &mut mapping, executor.runtime_mut());
        }

        // records output
        last_output = set_output;
    }

    Ok(mapped_output(output_data.map_data(), mapping, last_output))
}

/// Creates a mapped output from a mapping `Vec` and the last output
fn mapped_output(map_data: Option<&MapData>, mut mapping: Vec<String>, output: String) -> String {
    if let Some(map_data) = map_data {
        if map_data.reduce() {
            mapping.dedup();
        }

        mapping.join(&format!(" {} ", map_data.sep()))
    } else {
        output
    }
}

/// Creates a new mapping `Vec` based on the `MapType` and input
fn new_mapping(map_type: Option<MapType>, input: &str) -> Vec<String> {
    if map_type.is_some_and(|t| matches!(t, MapType::Final | MapType::FinalAndLogs)) {
        vec![input.to_string()]
    } else {
        Vec::new()
    }
}

/// Extends the mapping based on logs and rule set output
fn extend_mapping(map_type: MapType, output: &str, mapping: &mut Vec<String>, runtime: &mut AppRuntime) {
    if matches!(map_type, MapType::Logs | MapType::FinalAndLogs) {
        for (_msg, phones) in runtime.flush_logs() {
            mapping.push(phones);
        }
    }

    if matches!(map_type, MapType::Final | MapType::FinalAndLogs) {
        mapping.push(output.to_string());
    }
}

fn print_error(e: &cscsca::ScaError) {
    print!("{RED}");

    let error_type_msg = match e.error_type() {
        cscsca::ScaErrorType::Input => "Input",
        cscsca::ScaErrorType::Output => "Output",
        cscsca::ScaErrorType::Parse => "Syntax",
        cscsca::ScaErrorType::Application => "Application",
    };
    
    println!("{error_type_msg} Error{RESET}: {}", e.error_message());
    println!("Line {}: {}", e.line_number(), e.line());
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
    println!("{}", include_str!("../README.md"));
}

/// returns the template file
const fn template() -> &'static str {
    include_str!("assets/template.sca")
}

/// The logging `Runtime` for the cli application
/// with quiet (does not print PRINT statements)
/// and loud (prints PRINT statements)
#[derive(Debug)]
enum AppRuntime {
    Quiet(cscsca::LogRuntime),
    Loud(LogAndPrintRuntime),
}

impl AppRuntime {
    fn flush_logs(&mut self) -> Vec<(String, String)> {
        match self {
            Self::Quiet(logger) => logger.flush_logs(),
            Self::Loud(logger) => logger.flush_logs(),
        }
    }
}

impl cscsca::Runtime for AppRuntime {
    fn line_application_limit(&self) -> Option<cscsca::LineApplicationLimit> {
        Some(cscsca::LineApplicationLimit::default())
    }

    fn put_io(&mut self, msg: &str, phones: String) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Quiet(logger) => logger.put_io(msg, phones),
            Self::Loud(logger) => logger.put_io(msg, phones),
        }
    }

    fn on_start(&mut self) {
        match self {
            Self::Quiet(logger) => logger.on_start(),
            Self::Loud(logger) => logger.on_start(),
        }
    }

    fn on_end(&mut self) {
        match self {
            Self::Quiet(logger) => logger.on_end(),
            Self::Loud(logger) => logger.on_end(),
        }
    }
}