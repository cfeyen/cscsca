use std::{error::Error, fmt::Display, io::{stdin, stdout, Write}, time::Duration};

use crate::{
    color::{BLUE, RESET},
    applier::apply,
    ir::{tokenization_data::TokenizationData, tokenize_line_or_create_command, IrLine},
    keywords::{GET_AS_CODE_LINE_START, GET_LINE_START},
    phones::{build_phone_list, phone_list_to_string, Phone},
    rules::{build_rule, RuleLine},
    escaped_strings::EscapedString,
    ScaError,
};

pub const DEFAULT_LINE_APPLICATION_LIMIT: LineApplicationLimit = LineApplicationLimit::Attempts(10000);

/// Non rule commands executed by the runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Command {
    Print,
    Get,
    GetAsCode
}

/// A limit for how long a line can be executed for,
/// prevents infinite loops from being infinite
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineApplicationLimit {
    /// No limit on line application, allows infinite loops
    Unlimited,
    /// Maximum time allotted for line application
    Time(Duration),
    /// Maximum times an application attempt may be made by a line
    Attempts(usize),
}

/// A callback function for logging output
pub type PutFn = dyn FnMut(String) -> Result<(), Box<dyn Error>>;

/// A callback function for fetching input
pub type GetFn = dyn FnMut(String) -> Result<String, Box<dyn Error>>;

/// A context for appling sound changes
/// 
/// Determines the maximum amount a single line can apply changes for before being canceled,
/// and includes callbacks for:
///     - printing
///     - getting input
///     - limiting line application time/attempts
pub struct Runtime {
    /// The function called when logging
    io_put_fn: Box<PutFn>,
    /// The function called to fetch input
    io_get_fn: Box<GetFn>,
    /// The maximum amount of time allotted to apply changes to a line
    line_application_limit: LineApplicationLimit,
}

impl Default for Runtime {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    /// Creates a default runtime
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            io_put_fn: default_io_put_fn(),
            io_get_fn: default_io_get_fn(),
            line_application_limit: DEFAULT_LINE_APPLICATION_LIMIT,
        }
    }

    /// Creates a runtime with the given IO functions
    #[must_use]
    pub fn new_with_io(put: Box<PutFn>, get: Box<GetFn>) -> Self {
        Self {
            io_put_fn: put,
            io_get_fn: get,
            line_application_limit: DEFAULT_LINE_APPLICATION_LIMIT,
        }
    }

    /// Sets the `io_put_fn` callback for the runtime
    /// 
    /// Should send the printed message to an io device
    /// unless all outputs are to be retrieved from the print log after execution
    #[inline]
    pub fn set_io_put_fn(&mut self, callback: Box<PutFn>) -> &mut Self {
        self.io_put_fn = callback;
        self
    }

    /// Sets the `io_get_fn` callback for the runtime
    #[inline]
    pub fn set_io_get_fn(&mut self, callback: Box<GetFn>) -> &mut Self {
        self.io_get_fn = callback;
        self
    }

    /// Set the runtime's maximum application time per line
    #[inline]
    pub const fn set_line_application_limit(&mut self, limit: LineApplicationLimit) -> &mut Self {
        self.line_application_limit = limit;
        self
    }

    /// Gets the runtime's maximum application time per line
    #[inline]
    #[must_use]
    pub const fn get_line_application_limit(&self) -> LineApplicationLimit {
        self.line_application_limit
    }

    /// Applies rules to an input given the context of the runtime
    /// 
    /// ## Note
    /// This requires a mutable reference because the IO functions implement `FnMut`.
    /// Only the captures of those functions may be mutated
    /// 
    /// ## Errors
    /// Errors are the result of providing invalid code, failed io, or application timing out
    #[inline]
    pub fn apply(&mut self, input: &str, code: &str) -> Result<String, ScaError> {
        let escaped = EscapedString::from(input);

        let phones = build_phone_list(escaped.inner());

        self.apply_all_lines(phones, code)
    }

    /// Applies all lines, errors are returned as formated strings
    // ! must take ownership of phones so that the input sources can safely be freed to prevent memory leaks
    fn apply_all_lines<'s>(&mut self, mut phones: Vec<Phone<'s>>, code: &'s str) -> Result<String, ScaError> {
        // gets lines of code with line numbers,
        // and rule prepended so that escaped escape characters are properly outputted
        let lines = code.lines();
        let mut tokenization_data = TokenizationData::new();
        let mut line_num = 0;

        for line in lines {
            line_num += 1;
            if let Err(e) = self.apply_line(line, line_num, &mut phones, &mut tokenization_data) {
                drop(phones);
                // Safety: Since the output is a ScaError,
                // which owns all of its values, and phones is dropped,
                // no references remain to the sources buffer in `tokenization_data`
                unsafe { tokenization_data.free_sources() };
                return Err(e);
            }
        }

        let output = phone_list_to_string(&phones);

        drop(phones);
        // Safety: Since the output is a String,
        // which owns all of its values, and phones is dropped,
        // no references remain to the sources buffer in `tokenization_data`
        unsafe { tokenization_data.free_sources() };

        Ok(output)
    }

    /// Applies a line within the runtime, errers are returned as formated strings
    fn apply_line<'s>(&mut self, line: &'s str, line_num: usize, phones: &mut Vec<Phone<'s>>, tokenization_data: &mut TokenizationData<'s>) -> Result<(), ScaError> {
        // converts the line to ir
        let ir_line = tokenize_line_or_create_command(line, tokenization_data)
            .map_err(|e| ScaError::from_error(&e, line, line_num))?;

        match ir_line {
            IrLine::Cmd(cmd, args) => {
                self.handle_command(cmd, args, phones, tokenization_data)
                    .map_err(|e| ScaError::from_error(&*e, line, line_num))?;
            },
            // builds a rule from ir then applies it
            IrLine::Ir(_) => {
                let rule_line = build_rule(&ir_line)
                    .map_err(|e| ScaError::from_error(&e, line, line_num))?;

                drop(ir_line);

                if let RuleLine::Rule(rule) = rule_line {
                    apply(&rule, phones, self.line_application_limit)
                        .map_err(|e| ScaError::from_error(&e, line, line_num))?;
                }
            },
            // ignores empty lines
            IrLine::Empty => (),
        }

        Ok(())
    }

    /// Handles commands to the runtime
    fn handle_command<'s>(&mut self, cmd: Command, args: &'s str, phones: &[Phone], tokenization_data: &mut TokenizationData<'s>) -> Result<(), Box<dyn Error + 's>> {
        match cmd {
            // formats the message, calls the io_put_fn callback on it, then logs it
            Command::Print => {
                let msg = format!("{args} '{BLUE}{}{RESET}'", phone_list_to_string(phones));
                (self.io_put_fn)(msg)?;
            },
            // formats the message, calls the io_put_fn callback on it, then logs it
            Command::GetAsCode => {
                if let Some((name, msg)) = args.split_once(' ') {
                    let source = (self.io_get_fn)(msg.trim().to_string())?;

                    tokenization_data.set_variable_as_ir(name, source)?;
                } else {
                    return Err(Box::new(&GetFormatError));
                }
            },
            // formats the message, calls the io_put_fn callback on it, then logs it
            Command::Get => {
                if let Some((name, msg)) = args.split_once(' ') {
                    let source = (self.io_get_fn)(msg.trim().to_string())?;

                    tokenization_data.set_variable(name, &source);
                } else {
                    return Err(Box::new(&GetFormatError));
                }
            },
        }

        Ok(())
    }
}

/// Returns the default function for the runtime's `io_put_fn` callback
/// 
/// Prints to stdout
#[must_use]
fn default_io_put_fn() -> Box<PutFn> {
    Box::new(|msg| {
        println!("{msg}");
        Ok(())
    })
}

/// Returns the default function for the runtime's `io_put_fn` callback
/// 
/// Reads from stdin
#[must_use]
fn default_io_get_fn() -> Box<GetFn> {
    Box::new(|msg| {
        print!("{msg} ");
        let mut input = String::new();
        _ = stdout().flush();
        stdin().read_line(&mut input)?;

        let input = input.trim_end_matches(['\r', '\n']).to_string();

        Ok(input)
    })
}

#[derive(Debug)]
struct GetFormatError;

impl Error for GetFormatError {}

impl Display for GetFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid get format, should be {GET_LINE_START} 'var_name' 'msg' or {GET_AS_CODE_LINE_START} 'var_name' 'msg'")
    }
}