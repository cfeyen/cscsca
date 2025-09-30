use std::{error::Error, num::NonZero, time::Duration};

use crate::{
    applier::apply, await_io, io_fn, matcher::patterns::ir_to_patterns::RuleLine, phones::{phone_list_to_string, Phone}, RulelessScaError, ScaErrorType, ONE
};

use super::io_events::RuntimeIoEvent;

pub(crate) const DEFAULT_LINE_APPLICATION_LIMIT: LineApplicationLimit = LineApplicationLimit::Attempts(10000);

/// A limit for how long a line can be executed for,
/// prevents infinite loops from being infinite
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineApplicationLimit {
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

/// A trait that controls the runtime opperations of appying rules and IO
pub trait Runtime {
    /// Prints a message
    /// 
    /// # Errors
    /// Should only error on failed io
    /// 
    /// # Note
    /// This method should *not* be called outside of the `cscsca` crate
    #[io_fn]
    fn put_io(&mut self, msg: &str, phones: String) -> Result<(), Box<dyn Error>>;

    /// Called before applying a set of rules
    /// 
    /// Does nothing by default
    #[inline]
    fn on_start(&mut self) {}

    /// Called once applying a set of rules is complete
    /// 
    /// Does nothing by default
    #[inline]
    fn on_end(&mut self) {}

    /// The maximum limit for applying changes to a line
    #[inline]
    fn line_application_limit(&self) -> Option<LineApplicationLimit> {
        Some(DEFAULT_LINE_APPLICATION_LIMIT)
    }
}

/// An internal secondary trait that controls specifically how rules are applied
/// 
/// Is implemented on all implementers of `Runtime`
/// 
/// # Note
/// Default methods should not be overridden
pub(super) trait RuntimeApplier: Runtime {
    /// Applies changes for a single `RuleLine`
    #[io_fn]
    fn apply_line<'s: 'p, 'p>(&mut self, rule_line: &RuleLine<'s>, phones: &mut Vec<Phone<'p>>, line_num: NonZero<usize>) -> Result<(), RulelessScaError> {
        match rule_line {
            RuleLine::Empty => Ok(()),
            RuleLine::IoEvent(cmd) => await_io! {
                self.execute_runtime_command(cmd, phones, line_num)
            },
            RuleLine::Rule { rule, lines } => apply(rule, phones, self.line_application_limit())
                .map_err(|e| RulelessScaError::from_error(&e, ScaErrorType::Application, line_num, *lines))
        }
    }

    /// Executes a command at runtime
    #[io_fn]
    fn execute_runtime_command(&mut self, cmd: &RuntimeIoEvent<'_>, phones: &[Phone<'_>], line_num: NonZero<usize>) -> Result<(), RulelessScaError> {
        match cmd {
            RuntimeIoEvent::Print { msg } => {
                await_io! {
                    self.put_io(msg, phone_list_to_string(phones))
                }.map_err(|e| RulelessScaError::from_error(&*e, ScaErrorType::Output, line_num, ONE))
            }
        }
    }
}

impl<T: Runtime> RuntimeApplier for T {}

/// A basic `Runtime` that logs outputs to itself
/// 
/// Clears its logs before starting to apply a new set of rules
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRuntime {
    logs: Vec<(String, String)>,
    line_application_limit: Option<LineApplicationLimit>,
}

impl LogRuntime {
    /// Creates a new `LogRuntime`
    #[inline]
    #[must_use]
    pub const fn new(line_application_limit: Option<LineApplicationLimit>) -> Self {
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
    #[io_fn(impl)]
    fn put_io(&mut self, msg: &str, phones: String) -> Result<(), Box<dyn Error>> {
        self.logs.push((msg.to_string(), phones));
        Ok(())
    }

    #[inline]
    fn on_start(&mut self) {
        self.logs = Vec::new();
    }

    #[inline]
    fn line_application_limit(&self) -> Option<LineApplicationLimit> {
        self.line_application_limit
    }
}

impl Default for LogRuntime {
    fn default() -> Self {
        Self {
            logs: Vec::default(),
            line_application_limit: Some(DEFAULT_LINE_APPLICATION_LIMIT),
        }
    }
}