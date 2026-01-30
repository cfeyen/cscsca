#[cfg(test)]
mod tests;

use std::num::NonZero;

use crate::{
    ONE, ScaError, await_io, escaped_strings::EscapedString, executor::{
        build_line, getter::{ContextIoGetter, IoGetter}, runtime::{ContextRuntime, Runtime, RuntimeApplier}
    }, io_fn, ir::tokenization_data::TokenizationData, matcher::patterns::ir_to_patterns::RuleLine, phones::{build_phone_list, phone_list_to_string}
};

/// Builds all rules to a form that may be applied more easily within a given context
/// 
/// # Errors
/// Errors on invalid rules or failed io
#[io_fn]
pub fn build_rules_with_context<'s, G: ContextIoGetter>(rules: &'s str, getter: &mut G, ctx: G::InputContext) -> Result<AppliableRules<'s>, ScaError> {
    let tokenization_data = TokenizationData::new();
    
    await_io! { build_rules_with_tokenization_data_with_context(rules, tokenization_data, 0, getter, ctx) }
}

/// Builds all rules to a form that may be applied more easily
/// 
/// # Errors
/// Errors on invalid rules or failed io
#[io_fn]
#[inline]
pub fn build_rules<'s, G: IoGetter>(rules: &'s str, getter: &mut G) -> Result<AppliableRules<'s>, ScaError> {
    await_io! { build_rules_with_context(rules, getter, ()) }
}

/// Builds an `AppliableRules` struct from rules, pre-built tokenization data,
/// the number of lines that proceed the first line, and a `ContextIoGetter`
/// 
/// # Errors
/// Errors on invalid rules or failed io
#[io_fn]
fn build_rules_with_tokenization_data_with_context<'s, G: ContextIoGetter>(rules: &'s str, mut tokenization_data: TokenizationData<'s>, line_offset: usize, getter: &mut G, mut ctx: G::InputContext) -> Result<AppliableRules<'s>, ScaError> {
    let mut rule_lines = Vec::new();
    let mut lines = rules.lines().enumerate().map(|(line_num, line)| (unsafe { NonZero::new_unchecked(line_num + line_offset + 1) }, line));

    // prepares the getter to start fetching a new set of input
    getter.on_start();

    // builds each line
    while let Some((line_num, line)) = lines.next() {
        // builds the line and returns any errors
        let (rule_line, c) = match await_io! {
            build_line(line, &mut lines, line_num, &mut tokenization_data, getter, ctx)
        } {
            Ok(rule_line) => rule_line,
            Err(e) => {
                // signals to the getter that the rules are done being built
                getter.on_end();

                drop(rule_lines);
                // Safety: Since the output is a `ScaError`,
                // which owns all of its values, and `rule_lines` is dropped,
                // no references remain to the sources buffer in `tokenization_data`
                unsafe { tokenization_data.free_sources() };

                
                return Err(e.into_sca_error(rules.lines()));
            }
        };
        ctx = c;
        rule_lines.push(rule_line);
    }

    // signals to the getter that the rules are done being built
    getter.on_end();

    Ok(AppliableRules {
        lines: rules.lines().collect(),
        rules: rule_lines,
        tokenization_data,
    })
}

/// A set of rules reduced to an easily appliable form
/// that may be applied any number of times
#[derive(Debug)]
pub struct AppliableRules<'s> {
    /// References to each line of the input text (for error messages)
    lines: Vec<&'s str>,
    /// The built rules
    rules: Vec<RuleLine<'s>>,
    /// Data required to build rules, including raw pointers to input strings
    tokenization_data: TokenizationData<'s>,
}

impl<'s> AppliableRules<'s> {
    /// Applies all rules to the input using a runtime, errors are formatted as a string within a given context
    #[inline]
    #[io_fn]
    pub fn apply_with_context<R: ContextRuntime>(&self, input: &str, runtime: &mut R, ctx: R::OutputContext) -> String {
        await_io! {
            self.apply_fallible_with_context(input, runtime, ctx)
        }.unwrap_or_else(|e| e.to_string())
    }

    /// Applies all rules to the input using a runtime, errors are formatted as a string
    #[io_fn]
    #[inline]
    pub fn apply<R: Runtime>(&self, input: &str, runtime: &mut R) -> String {
        await_io! {
            self.apply_with_context(input, runtime, ())
        }
    }

    /// Applies all rules to the input using a runtime within a given context
    /// 
    /// # Errors
    /// Errors on invalid rules, application that takes too long, and failed io
    #[io_fn]
    pub fn apply_fallible_with_context<R: ContextRuntime>(&self, input: &str, runtime: &mut R, mut ctx: R::OutputContext) -> Result<String, ScaError> {
        let escaped_input = EscapedString::from(input);
        let mut phones = build_phone_list(escaped_input.as_escaped_str());

        let mut line_num = ONE;

        // prepares the runtime for a new set of applications
        runtime.on_start();

        // applies rules
        for rule_line in &self.rules {
            match await_io! { runtime.apply_line(ctx, rule_line, &mut phones, line_num) } {
                Ok(c) => ctx = c,
                Err(e) => {
                    // signals to the runtime that execution is complete
                    runtime.on_end();

                    return Err(e.into_sca_error(self.lines.iter().copied()));
                }
            }

            line_num = line_num.saturating_add(rule_line.lines().get());
        }

        // signals to the runtime that execution is complete
        runtime.on_end();

        Ok(phone_list_to_string(&phones))
    }

    /// Applies all rules to the input using a runtime
    /// 
    /// # Errors
    /// Errors on invalid rules, application that takes too long, and failed io
    #[io_fn]
    #[inline]
    pub fn apply_fallible<R: Runtime>(&self, input: &str, runtime: &mut R) -> Result<String, ScaError> {
        await_io! {
            self.apply_fallible_with_context(input, runtime, ())
        }
    }


    /// Extends appliable rules with new rules source within a given context
    /// 
    /// # Errors
    /// Errors on invalid rules or failed io
    /// 
    /// leaves `self` unchanged on error
    #[io_fn]
    pub fn extend_with_context<G: ContextIoGetter>(&mut self, next_rules: &'s str, getter: &mut G, ctx: G::InputContext) -> Result<(), ScaError> {
        let line_offset = self.rules.iter().fold(0, |acc, rule_line| acc + rule_line.lines().get());
        let tokenization_data = self.tokenization_data.with_inserts();
        
        let mut new_appliable = await_io! {
            build_rules_with_tokenization_data_with_context(next_rules, tokenization_data, line_offset, getter, ctx)
        }?;

        self.lines.append(&mut new_appliable.lines);
        self.rules.append(&mut new_appliable.rules);
        std::mem::swap(&mut self.tokenization_data, &mut new_appliable.tokenization_data);

        // Saftey: it is safe to drop `new_appliable`
        // because all of its sources have been moved to `self`
        // `new_appliable` is dropped before `self`
        unsafe { self.tokenization_data.take_sources_from(&mut new_appliable.tokenization_data) };

        Ok(())
    }

    /// Extends appliable rules with new rules source
    /// 
    /// # Errors
    /// Errors on invalid rules or failed io
    /// 
    /// leaves `self` unchanged on error
    #[io_fn]
    #[inline]
    pub fn extend<G: IoGetter>(&mut self, next_rules: &'s str, getter: &mut G) -> Result<(), ScaError> {
        await_io! {
            self.extend_with_context(next_rules, getter, ())
        }
    }

    /// Returns a copy of the source rules
    #[must_use]
    pub fn get_rules(&self) -> String {
        self.lines.join("\n")
    }
}

impl Drop for AppliableRules<'_> {
    fn drop(&mut self) {
        for source in self.tokenization_data.sources() {
            // Safety: `AppliableRules` should never be cloned
            // or leak references to the IO sources
            // ! this must be invarient and maintained within the `AppliableRules` API
            let ptr = source.cast_mut();
            unsafe { drop(Box::from_raw(ptr)); }
        }
    }
}
