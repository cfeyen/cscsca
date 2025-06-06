pub mod runtime;
pub mod getter;
pub mod appliable_rules;
pub(crate) mod commands;

#[cfg(test)]
mod tests;

use crate::{
    escaped_strings::EscapedString, 
    ir::{tokenization_data::{TokenizationData}, tokenize_line_or_create_command, IrLine},
    phones::{build_phone_list, phone_list_to_string},
    rules::{build_rule, RuleLine},
    ScaError,
};
use commands::Command;
use runtime::{Runtime, RuntimeApplier};
use getter::{IoGetter, ComptimeCommandExecuter};

/// An executer that contains both an `IoGetter` and a `Runtime`
/// 
/// Compiles then applies one line at a time
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
    pub fn apply(&mut self, input: &str, rules: &str) -> String {
        self.apply_fallible(input, rules)
            .unwrap_or_else(|e| e.to_string())
    }

    /// Applies the rules to the input
    /// 
    /// ## Errors
    /// Errors on invalid rules, application that takes too long, and failed io
    pub fn apply_fallible(&mut self, input: &str, rules: &str) -> Result<String, ScaError> {
        let escaped = EscapedString::from(input);
        let mut phones = build_phone_list(escaped.as_escaped_str());

        let lines = rules.lines();
        let mut tokenization_data = TokenizationData::new();
        let mut line_num = 0;

        for line in lines {
            line_num += 1;

            let application_result = compile_line(line, line_num, &mut tokenization_data, &mut self.getter)
                .map(|rule_line| self.runtime.apply_line(&rule_line, &mut phones, line, line_num));

            if let Err(e) | Ok(Err(e)) = application_result {
                drop(phones);
                // Safety: Since the output is a ScaError,
                // which owns all of its values, and `phones` is dropped,
                // no references remain to the sources buffer in `tokenization_data`
                unsafe { tokenization_data.free_sources() };
                return Err(e);
            }
        }

        let output = phone_list_to_string(&phones);

        drop(phones);
        // Safety: Since the output is a String,
        // which owns all of its values, and `phones` is dropped,
        // no references remain to the sources buffer in `tokenization_data`
        unsafe { tokenization_data.free_sources() };

        Ok(output)
    }
}

/// Compiles a line from a string to a `RuleLine`
fn compile_line<'s, G>(line: &'s str, line_num: usize, tokenization_data: &mut TokenizationData<'s>, getter: &mut G) -> Result<RuleLine<'s>, ScaError>
where
    G: IoGetter
{
    let ir_line = tokenize_line_or_create_command(line, tokenization_data)
        .map_err(|e| ScaError::from_error(&e, line, line_num))?;

    match ir_line {
        IrLine::Cmd(Command::ComptimeCommand(cmd)) => {
            getter.run_compile_time_command(&cmd, tokenization_data, line, line_num)?;
            Ok(RuleLine::Empty)
        },
        // builds a rule from ir
        ir_line =>
            build_rule(ir_line)
                .map_err(|e| ScaError::from_error(&e, line, line_num)),
    }
}