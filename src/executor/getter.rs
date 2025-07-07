use std::{error::Error, io::{self, Write}};

use crate::{
    ir::tokenization_data::TokenizationData,
    ScaError,
    await_io,
    io_fn,
};
use super::io_events::{TokenizerIoEvent, GetType};

/// A trait that controls how input is fetched when building rules
pub trait IoGetter {
    /// Gets input
    /// 
    /// # Errors
    /// Should only error on failed io
    /// 
    /// # Note:
    /// This method should *not* be called outside of the `cscsca` crate
    #[io_fn]
    fn get_io(&mut self, msg: &str) -> Result<String, Box<dyn Error>>;

    /// Called before building a set of rules
    /// 
    /// Does nothing by default
    #[inline]
    fn on_start(&mut self) {}

    /// Called once a set of rules is done being building
    /// 
    /// Does nothing by default
    #[inline]
    fn on_end(&mut self) {}
}


/// An internal secondary trait that controls specifically how build time commands are executed
/// 
/// Is implemented on all implementers of `IoGetter`
/// 
/// # Note
/// Default methods should not be overridden
pub(super) trait ComptimeCommandExecuter: IoGetter {
    /// Runs a command at build time
    #[io_fn]
    fn run_build_time_command<'s>(&mut self, cmd: &TokenizerIoEvent<'s>, tokenization_data: &mut TokenizationData<'s>, line: &str, line_num: usize) -> Result<(), ScaError> {
        match cmd {
            TokenizerIoEvent::Get { get_type, var, msg } => {
                let input = await_io! {
                    self.get_io(msg)
                }.map_err(|e| ScaError::from_io_error(&*e, line, line_num))?;

                match get_type {
                    GetType::Phones => tokenization_data.set_variable(var, &input),
                    GetType::Code => tokenization_data.set_variable_as_ir(var, input)
                        .map_err(|e| ScaError::from_error(&e, line, line_num))?,
                }

                Ok(())
            }
        }
    }
}

impl<T: IoGetter> ComptimeCommandExecuter for T {}

/// A basic `IoGetter` that get input from standard input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CliGetter;

impl CliGetter {
    /// Creates a new `CliGetter`
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl IoGetter for CliGetter {
    #[inline]
    #[io_fn]
    fn get_io(&mut self, msg: &str) -> Result<String, Box<dyn Error>> {
        print!("{msg} ");
        let mut buffer = String::new();
        _ = io::stdout().flush();
        io::stdin().read_line(&mut buffer)?;
        Ok(buffer.trim().to_string())
    }
}