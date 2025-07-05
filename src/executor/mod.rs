pub mod runtime;
pub mod getter;
pub mod appliable_rules;
pub(crate) mod io_events;

#[cfg(test)]
mod tests;

use crate::{
    escaped_strings::EscapedString, 
    ir::{tokenization_data::{TokenizationData}, tokenize_line_or_create_command, IrLine},
    phones::{build_phone_list, phone_list_to_string},
    rules::{build_rule, RuleLine},
    ScaError,
    await_io,
    io_fn,
};
use io_events::IoEvent;
use runtime::{Runtime, RuntimeApplier};
use getter::{IoGetter, ComptimeCommandExecuter};

/// An executer that contains both an `IoGetter` and a `Runtime`
/// 
/// Builds then applies one line at a time
#[derive(Debug, Clone, Copy)]
pub struct LineByLineExecuter<R: Runtime, G: IoGetter> {
    runtime: R,
    getter: G,
}

impl<R: Runtime, G: IoGetter> LineByLineExecuter<R, G> {
    /// Creates a new `LineByLineExecuter`
    #[inline]
    pub const fn new(runtime: R, getter: G) -> Self {
        Self {
            runtime,
            getter,
        }
    }

    /// Returns a reference to the runtime
    #[inline]
    pub const fn runtime(&self) -> &R {
        &self.runtime
    }

    /// Returns a mutable reference to the runtime
    #[inline]
    pub const fn runtime_mut(&mut self) -> &mut R {
        &mut self.runtime
    }

    /// Returns a reference to the getter
    #[inline]
    pub const fn getter(&self) -> &G {
        &self.getter
    }

    /// Returns a mutable reference to the getter
    #[inline]
    pub const fn getter_mut(&mut self) -> &mut G {
        &mut self.getter
    }

    /// Applies the rules to the input, all errors are a formatted string
    #[inline]
    #[io_fn]
    pub fn apply(&mut self, input: &str, rules: &str) -> String {
        await_io! {
            self.apply_fallible(input, rules)
        }.unwrap_or_else(|e| e.to_string())
    }

    /// Applies the rules to the input
    /// 
    /// ## Errors
    /// Errors on invalid rules, application that takes too long, and failed io
    #[io_fn]
    pub fn apply_fallible(&mut self, input: &str, rules: &str) -> Result<String, ScaError> {
        let escaped = EscapedString::from(input);
        let mut phones = build_phone_list(escaped.as_escaped_str());

        let lines = rules.lines();
        let mut tokenization_data = TokenizationData::new();
        let mut line_num = 0;

        // prepares the runtime and getter for a new set of applications
        self.getter.on_start();
        self.runtime.on_start();

        // builds and applies rules line by line
        for line in lines {
            line_num += 1;

            // builds and attempts to apply the rules
            let application_result = match await_io! {
                build_line(line, line_num, &mut tokenization_data, &mut self.getter)
            } {
                Ok(rule_line) => Ok(await_io! {
                    self.runtime.apply_line(&rule_line, &mut phones, line, line_num)
                }),
                Err(e) => Err(e),
            };

            // handles errors
            if let Err(e) | Ok(Err(e)) = application_result {
                // signals to the runtime and getter that execution is complete
                self.getter.on_end();
                self.runtime.on_end();

                drop(phones);
                // Safety: Since the output is a ScaError,
                // which owns all of its values, and `phones` is dropped,
                // no references remain to the sources buffer in `tokenization_data`
                unsafe { tokenization_data.free_sources() };
                return Err(e);
            }
        }

        // signals to the runtime and getter that execution is complete
        self.getter.on_end();
        self.runtime.on_end();

        let output = phone_list_to_string(&phones);

        drop(phones);
        // Safety: Since the output is a String,
        // which owns all of its values, and `phones` is dropped,
        // no references remain to the sources buffer in `tokenization_data`
        unsafe { tokenization_data.free_sources() };

        Ok(output)
    }
}

/// Builds a line from a string to a `RuleLine`
#[io_fn]
fn build_line<'s, G>(line: &'s str, line_num: usize, tokenization_data: &mut TokenizationData<'s>, getter: &mut G) -> Result<RuleLine<'s>, ScaError>
where
    G: IoGetter
{
    let ir_line = tokenize_line_or_create_command(line, tokenization_data)
        .map_err(|e| ScaError::from_error(&e, line, line_num))?;

    match ir_line {
        IrLine::IoEvent(IoEvent::Tokenizer(cmd)) => {
            await_io! { getter.run_build_time_command(&cmd, tokenization_data, line, line_num) }?;
            Ok(RuleLine::Empty)
        },
        // builds a rule from ir
        ir_line =>
            build_rule(ir_line)
                .map_err(|e| ScaError::from_error(&e, line, line_num)),
    }
}