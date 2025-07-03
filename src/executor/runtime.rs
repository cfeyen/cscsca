use std::{error::Error, time::Duration};
use crate::{
    applier::apply,
    color::{BLUE, RESET},
    phones::{phone_list_to_string, Phone},
    rules::RuleLine,
    ScaError,
};
use super::commands::RuntimeCommand;

pub(crate) const DEFAULT_LINE_APPLICATION_LIMIT: LineApplicationLimit = LineApplicationLimit::Attempts(10000);

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

impl Default for LineApplicationLimit {
    fn default() -> Self {
        DEFAULT_LINE_APPLICATION_LIMIT
    }
}

/// A trait that controls the runtime opperations of appying rules
pub trait Runtime {
    /// Prints a message
    /// 
    /// ## Errors
    /// Should only error on failed io
    /// 
    /// ## Note:
    /// This method should *not* be called outside of the `cscsca` crate
    fn put_io(&mut self, msg: &str, phones: String) -> Result<(), Box<dyn Error>>;

    /// Called before applying a set of rules
    /// 
    /// Does nothing by default
    fn on_start(&mut self) {}

    /// Called once applying a set of rules is complete
    /// 
    /// Does nothing by default
    fn on_end(&mut self) {}

    /// The maximum limit for applying changes to a line
    fn line_application_limit(&self) -> LineApplicationLimit;
}

/// An internal secondary trait that controls specifically how rules are applied
/// 
/// Is implemented on all implementers of `Runtime`
/// 
/// ## Note
/// Default methods should not be overridden
pub(super) trait RuntimeApplier: Runtime {
    /// Applies changes for a single `RuleLine`
    fn apply_line<'s>(&mut self, rule_line: &RuleLine<'s>, phones: &mut Vec<Phone<'s>>, line: &str, line_num: usize) -> Result<(), ScaError> {
        match rule_line {
            RuleLine::Empty => Ok(()),
            RuleLine::Cmd(cmd) => self.execute_runtime_command(cmd, phones, line, line_num),
            RuleLine::Rule(rule) => apply(rule, phones, self.line_application_limit())
                .map_err(|e| ScaError::from_error(&e, line, line_num))
        }
    }

    /// Executes a command at runtime
    fn execute_runtime_command(&mut self, cmd: &RuntimeCommand, phones: &[Phone], line: &str, line_num: usize) -> Result<(), ScaError> {
        match cmd {
            RuntimeCommand::Print { msg } => {
                self.put_io(msg, phone_list_to_string(phones))
                    .map_err(|e| ScaError::from_io_error(&*e, line, line_num))
            }
        }
    }
}

impl<T: Runtime> RuntimeApplier for T {}

/// A basic `Runtime` that prints to standard output
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct CliRuntime {
    line_application_limit: LineApplicationLimit,
}

impl CliRuntime {
    /// Creates a new `CliRuntime`
    #[inline]
    #[must_use]
    pub const fn new(line_application_limit: LineApplicationLimit) -> Self {
        Self { line_application_limit }
    }
}

impl Runtime for CliRuntime {
    #[inline]
    fn line_application_limit(&self) -> LineApplicationLimit {
        self.line_application_limit
    }

    #[inline]
    fn put_io(&mut self, msg: &str, phones: String) -> Result<(), Box<dyn Error>> {
        println!("{msg} '{BLUE}{phones}{RESET}'");
        Ok(())
    }
}

/// A basic `Runtime` that logs outputs to itself
/// 
/// Clears its logs before starting to apply a new set of rules
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LogRuntime {
    logs: Vec<(String, String)>,
    line_application_limit: LineApplicationLimit,
}

impl LogRuntime {
    /// Creates a new `LogRuntime`
    #[inline]
    #[must_use]
    pub const fn new(line_application_limit: LineApplicationLimit) -> Self {
        Self {
            logs: Vec::new(),
            line_application_limit,
        }
    }

    /// Returns the logs
    #[inline]
    #[must_use]
    pub fn logs(&self) -> &[(String, String)] {
        &self.logs
    }

    /// Returns the logs and replaces them with empty logs
    #[inline]
    pub fn flush_logs(&mut self) -> Vec<(String, String)> {
        let mut log_replacement = Vec::new();

        std::mem::swap(&mut log_replacement, &mut self.logs);

        log_replacement
    }
}

impl Runtime for LogRuntime {
    fn put_io(&mut self, msg: &str, phones: String) -> Result<(), Box<dyn Error>> {
        self.logs.push((msg.to_string(), phones));
        Ok(())
    }

    #[inline]
    fn on_start(&mut self) {
        self.logs = Vec::new();
    }

    #[inline]
    fn line_application_limit(&self) -> LineApplicationLimit {
        self.line_application_limit
    }
}

/// A basic `Runtime` that logs outputs to itself and prints its logs to standard output
/// 
/// Clears its logs before starting to apply a new set of rules
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LogAndPrintRuntime(LogRuntime);

impl LogAndPrintRuntime {
    /// Returns the logs
    #[must_use]
    pub fn logs(&self) -> &[(String, String)] {
        self.0.logs()
    }

    /// Returns the logs and replaces them with empty logs
    pub fn flush_logs(&mut self) -> Vec<(String, String)> {
        self.0.flush_logs()
    }
}

impl Runtime for LogAndPrintRuntime {
    #[inline]
    fn line_application_limit(&self) -> LineApplicationLimit {
        self.0.line_application_limit()
    }

    #[inline]
    fn put_io(&mut self, msg: &str, phones: String) -> Result<(), Box<dyn Error>> {
        println!("{msg} '{BLUE}{phones}{RESET}'");
        self.0.put_io(msg, phones)
    }

    #[inline]
    fn on_start(&mut self) {
        self.0.on_start();
    }
}