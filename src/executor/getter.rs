use std::num::NonZero;

use crate::{
    await_io, io_fn, ir::tokenization_data::TokenizationData, RulelessScaError, ScaErrorType, ONE
};
use super::io_events::{TokenizerIoEvent, GetType};

/// A trait that controls how input is fetched when building rules with a given context
pub trait ContextIoGetter {
    /// A context that can be passed to the getter when fetching input
    type InputContext;

    /// Gets input
    /// 
    /// # Errors
    /// Should only error on failed io
    /// 
    /// # Note
    /// This method should *not* be called outside of the `cscsca` crate
    #[io_fn]
    fn get_io(&mut self, context: &mut Self::InputContext, msg: &str) -> Result<String, String>;

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

impl<T: IoGetter> ContextIoGetter for T {
    type InputContext = ();

    #[io_fn(impl)]
    #[inline]
    fn get_io(&mut self, (): &mut Self::InputContext, msg: &str) -> Result<String, String> {
        await_io! { IoGetter::get_io(self, msg) }
    }

    #[inline]
    fn on_start(&mut self) {
        IoGetter::on_start(self);
    }

    #[inline]
    fn on_end(&mut self) {
        IoGetter::on_end(self);
    }
}

/// A trait that controls how input is fetched when building rules
/// 
/// Auto-implements `ContextIoGetter`<`InputContext`=`()`>
pub trait IoGetter {
    /// Gets input
    /// 
    /// # Errors
    /// Should only error on failed io
    /// 
    /// # Note
    /// This method should *not* be called outside of the `cscsca` crate
    #[io_fn]
    fn get_io(&mut self, msg: &str) -> Result<String, String>;

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
pub(super) trait ComptimeCommandExecuter: ContextIoGetter {
    /// Runs a command at build time
    #[io_fn]
    fn run_build_time_command<'s>(&mut self, cxt: &mut Self::InputContext, cmd: &TokenizerIoEvent<'s>, tokenization_data: &mut TokenizationData<'s>, line_num: NonZero<usize>) -> Result<(), RulelessScaError> {
        match cmd {
            TokenizerIoEvent::Get { get_type, var, msg } => {
                let input = await_io! {
                    self.get_io(cxt, msg)
                }.map_err(|e| RulelessScaError::from_error_message(e, ScaErrorType::Input, line_num, ONE))?;

                match get_type {
                    GetType::Phones => tokenization_data.set_variable(var, &input),
                    GetType::Code => tokenization_data.set_variable_as_ir(var, input)
                        .map_err(|e| RulelessScaError::from_error(&e, ScaErrorType::Input, line_num, ONE))?,
                }

                Ok(())
            }
        }
    }
}

impl<T: ContextIoGetter> ComptimeCommandExecuter for T {}