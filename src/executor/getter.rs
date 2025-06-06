use std::{error::Error, io::{self, Write}};
use crate::{ir::tokenization_data::TokenizationData, ScaError};
use super::commands::{ComptimeCommand, GetType};

pub trait IoGetter {
    /// Gets input
    /// 
    /// ## Errors
    /// Should only error on failed io
    /// 
    /// ## Note:
    /// This method should *not* be called outside of the `cscsca` crate
    fn get_io(&mut self, msg: &str) -> Result<String, Box<dyn Error>>;
}


/// An internal secondary trait that controls specifically how compile time commands are executed
/// 
/// Is implemented on all implementers of `IoGetter`
/// 
/// ## Note
/// Default methods should not be overridden
pub(super) trait ComptimeCommandExecuter: IoGetter {
    /// Runs a command at compile time
    fn run_compile_time_command<'s>(&mut self, cmd: &ComptimeCommand<'s>, tokenization_data: &mut TokenizationData<'s>, line: &str, line_num: usize) -> Result<(), ScaError> {
        match cmd {
            ComptimeCommand::Get { get_type, var, msg } => {
                let input = self.get_io(msg)
                    .map_err(|e| ScaError::from_io_error(&*e, line, line_num))?;

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
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl IoGetter for CliGetter {
    #[inline]
    fn get_io(&mut self, msg: &str) -> Result<String, Box<dyn Error>> {
        print!("{msg} ");
        let mut buffer = String::new();
        _ = io::stdout().flush();
        io::stdin().read_line(&mut buffer)?;
        Ok(buffer.trim().to_string())
    }
}