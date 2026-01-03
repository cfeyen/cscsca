pub mod runtime;
pub mod getter;
pub mod appliable_rules;
pub(crate) mod io_events;

#[cfg(test)]
mod tests;

use std::num::NonZero;

use crate::{
    ONE, RulelessScaError, ScaError, ScaErrorType, await_io, escaped_strings::EscapedString,
    executor::{getter::ContextIoGetter, runtime::ContextRuntime},
    io_fn,
    ir::{IrLine, tokenization_data::TokenizationData, tokenizer::tokenize_line_or_create_command},
    matcher::patterns::ir_to_patterns::{RuleLine, build_rule},
    phones::{build_phone_list, phone_list_to_string}
};

use io_events::IoEvent;
use runtime::{Runtime, RuntimeApplier};
use getter::{IoGetter, ComptimeCommandExecuter};

/// An executer that contains both an `IoGetter` and a `Runtime`
/// 
/// Builds then applies one line at a time
#[derive(Debug, Clone, Copy, Default)]
pub struct LineByLineExecuter<R: ContextRuntime, G: ContextIoGetter> {
    runtime: R,
    getter: G,
}


impl<R: Runtime, G: IoGetter> LineByLineExecuter<R, G> {
/// Applies the rules to the input, all errors are a formatted string
    #[inline]
    #[io_fn]
    pub fn apply(&mut self, input: &str, rules: &str) -> String {
        await_io! {
            self.apply_with_contexts(input, rules, &mut (), &mut ())
        }
    }

    /// Applies the rules to the input
    /// 
    /// # Errors
    /// Errors on invalid rules, application that takes too long, and failed io
    #[io_fn]
    pub fn apply_fallible(&mut self, input: &str, rules: &str) -> Result<String, ScaError> {
        await_io! {
            self.apply_fallible_with_contexts(input, rules, &mut (), &mut ())
        }
    }
}

impl<R: ContextRuntime, G: ContextIoGetter> LineByLineExecuter<R, G> {
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

    /// Applies the rules to the input within the given contexts, all errors are a formatted string
    #[inline]
    #[io_fn]
    pub fn apply_with_contexts(&mut self, input: &str, rules: &str, ocxt: &mut R::OutputContext, icxt: &mut G::InputContext) -> String {
        await_io! {
            self.apply_fallible_with_contexts(input, rules, ocxt, icxt)
        }.unwrap_or_else(|e| e.to_string())
    }

    /// Applies the rules to the input within the given contexts
    /// 
    /// # Errors
    /// Errors on invalid rules, application that takes too long, and failed io
    #[io_fn]
    pub fn apply_fallible_with_contexts(&mut self, input: &str, rules: &str, ocxt: &mut R::OutputContext, icxt: &mut G::InputContext) -> Result<String, ScaError> {
        let escaped = EscapedString::from(input);
        let mut phones = build_phone_list(escaped.as_escaped_str());

        let mut lines = rules.lines().enumerate().map(|(line_num, line)| (unsafe { NonZero::new_unchecked(line_num + 1) }, line));
        let mut tokenization_data = TokenizationData::new();

        // prepares the runtime and getter for a new set of applications
        self.getter.on_start();
        self.runtime.on_start();

        // builds and applies rules line by line
        while let Some((line_num, line)) = lines.next() {
            // builds and attempts to apply the rules
            let application_result = match await_io! {
                build_line(line, &mut lines, line_num, &mut tokenization_data, &mut self.getter, icxt)
            } {
                Ok(rule_line) => Ok(await_io! {
                    self.runtime.apply_line(ocxt, &rule_line, &mut phones, line_num)
                }),
                Err(e) => Err(e),
            };

            // handles errors
            if let Err(e) | Ok(Err(e)) = application_result {
                // signals to the runtime and getter that execution is complete
                self.getter.on_end();
                self.runtime.on_end();

                drop(phones);
                // Safety: Since the output is a `ScaError`,
                // which owns all of its values, and `phones` is dropped,
                // no references remain to the sources buffer in `tokenization_data`
                unsafe { tokenization_data.free_sources() };

                return Err(e.into_sca_error(rules.lines()));
            }
        }

        // signals to the runtime and getter that execution is complete
        self.getter.on_end();
        self.runtime.on_end();

        let output = phone_list_to_string(&phones);

        drop(phones);
        // Safety: Since the output is a `String`,
        // which owns all of its values, and `phones` is dropped,
        // no references remain to the sources buffer in `tokenization_data`
        unsafe { tokenization_data.free_sources() };

        Ok(output)
    }
}

/// Builds a line from a string to a `RuleLine`
#[io_fn]
fn build_line<'s, G: ContextIoGetter>(line: &'s str, rem_lines: &mut impl Iterator<Item = (NonZero<usize>, &'s str)>, line_num: NonZero<usize>, tokenization_data: &mut TokenizationData<'s>, getter: &mut G, cxt: &mut G::InputContext) -> Result<RuleLine<'s>, RulelessScaError> {
    let mut line_count = ONE;

    let ir_line = tokenize_line_or_create_command(line, &mut rem_lines.map(|(_, line)| {
        line_count = line_count.saturating_add(1);
        line
    }), tokenization_data)
        .map_err(|e| RulelessScaError::from_error(&e, ScaErrorType::Parse, line_num, line_count))?;

    match ir_line {
        IrLine::IoEvent(IoEvent::Tokenizer(cmd)) => {
            await_io! { getter.run_build_time_command(cxt, &cmd, tokenization_data, line_num) }?;
            Ok(RuleLine::Empty { lines: line_count })
        },
        // builds a rule from ir
        ir_line =>
            build_rule(ir_line)
                .map_err(|e| RulelessScaError::from_error(&e, ScaErrorType::Parse, line_num, line_count)),
    }
}