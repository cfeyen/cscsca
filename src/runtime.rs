use std::{collections::HashMap, error::Error, time::Duration};

use crate::{applier::apply, build_phone_list, colors::{BLUE, RESET}, format_error, phone_list_to_string, phones::Phone, rules::{build_rule, RuleLine}, runtime_cmds::{PrintLog, RuntimeCmd}, tokens::{ir::IrToken, token_checker::check_token_line, tokenize_line_or_create_runtime_command, IrLine}};

pub const DEFAULT_MAX_APPLICATION_TIME: Duration = Duration::from_millis(100);

/// A callback function for logging
/// 
/// Should send the printed message to an io device
/// unless all outputs are to be retrieved from the print log after execution
pub type PutFn = Box<fn(&str) -> Result<(), Box<dyn Error>>>;

/// Returns the default function for the runtime's io_put_fn callback
#[inline]
pub fn default_io_put_fn() -> PutFn {
    Box::new(|msg| {
        println!("{msg}");
        Ok(())
    })
}

/// A context for appling sound changes
/// 
/// Determines the maximum amount a single line can apply changes for before being canceled,
/// and includes callbacks for:
///     - printing
pub struct Runtime {
    /// The function called when logging
    ///
    /// Should send the printed message to an io device
    /// unless all outputs are to be retrieved from the print log after execution
    io_put_fn: PutFn,
    /// The maximum amount of time allotted to apply changes to a line
    max_line_application_time: Duration,
}

impl Default for Runtime {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    #[inline]
    /// Creates a default runtime
    pub fn new() -> Self {
        Self {
            io_put_fn: default_io_put_fn(),
            max_line_application_time: DEFAULT_MAX_APPLICATION_TIME,
        }
    }

    /// Sets the `io_put_fn` callback for the runtime
    /// 
    /// Should send the printed message to an io device
    /// unless all outputs are to be retrieved from the print log after execution
    #[inline]
    pub fn set_io_put_fn(&mut self, callback: PutFn) -> &mut Self {
        self.io_put_fn = callback;
        self
    }

    /// Returns the runtime's maximum application time per line
    #[inline]
    pub const fn max_line_application_time(&self) -> &Duration {
        &self.max_line_application_time
    }

    /// Set the runtime's maximum application time per line
    #[inline]
    pub const fn set_max_line_application_time(&mut self, time: Duration) -> &mut Self {
        self.max_line_application_time = time;
        self
    }

    /// Applies rules to an input given the context of the runtime, errors are returned as formated strings
    pub fn apply(&mut self, input: &str, code: &str) -> (Result<String, String>, PrintLog) {
        let mut log = PrintLog::new();

        (self.apply_all_lines(input, code, &mut log), log)
    }

    /// Applies all lines, errors are returned as formated strings
    fn apply_all_lines(&mut self, input: &str, code: &str, print_log: &mut PrintLog) -> Result<String, String> {
        let lines = code
            .lines()
            .enumerate()
            .map(|(num, line)| (num + 1, line));

        let mut phones = build_phone_list(input);
        let mut definitions = HashMap::new();

        for (line_num, line) in lines {
            self.apply_line(line, line_num, &mut phones, print_log, &mut definitions)?;
        }

        Ok(phone_list_to_string(&phones))
    }

    /// Applies a line within the runtime, errers are returned as formated strings
    fn apply_line<'s>(&mut self, line: &'s str, line_num: usize, phones: &mut Vec<Phone<'s>>, print_log: &mut PrintLog, definitions: &mut HashMap<&'s str, Vec<IrToken<'s>>>) -> Result<(), String> {
        // converts the line to ir
        let ir_line = tokenize_line_or_create_runtime_command(line, definitions)
            .map_err(|e| format_error(&e, line, line_num))?;

        match ir_line {
            // formats the message, calls the io_put_fn callback on it, then logs it
            IrLine::Cmd(RuntimeCmd::Print, args) => {
                let msg = format!("{args} '{BLUE}{}{RESET}'", phone_list_to_string(phones));
                (self.io_put_fn)(&msg)
                    .map_err(|e| format_error(&*e, line, line_num))?;
                print_log.log(msg);
            },
            // checks ir, builds a rule, and applies it
            IrLine::Ir(_) => {
                check_token_line(&ir_line)
                    .map_err(|e| format_error(&e, line, line_num))?;

                let rule_line = build_rule(&ir_line)
                    .map_err(|e| format_error(&e, line, line_num))?;

                if let RuleLine::Rule(rule) = rule_line {
                    apply(&rule, phones, &self.max_line_application_time)
                        .map_err(|e| format_error(&e, line, line_num))?;
                }
            },
            // ignores empty lines
            IrLine::Empty => (),
        }

        Ok(())
    }
}