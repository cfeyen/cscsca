#[cfg(test)]
mod tests;

use std::num::NonZero;

use crate::{
    await_io, escaped_strings::EscapedString, executor::{
        build_line, getter::IoGetter, runtime::{Runtime, RuntimeApplier}
    }, io_fn, ir::tokenization_data::TokenizationData, phones::{build_phone_list, phone_list_to_string}, rules::RuleLine, ScaError, ONE
};

/// Builds all rules to a form that may be applied more easily
/// 
/// # Errors
/// Errors on invalid rules or failed io
#[io_fn]
pub fn build_rules<'s, G: IoGetter>(rules: &'s str, getter: &mut G) -> Result<AppliableRules<'s>, ScaError> {
    let mut rule_lines = Vec::new();
    let mut tokenization_data = TokenizationData::new();
    let mut lines = rules.lines().enumerate().map(|(line_num, line)| (unsafe { NonZero::new_unchecked(line_num + 1) }, line));

    // prepares the getter to start fetching a new set of input
    getter.on_start();

    // builds each line
    while let Some((line_num, line)) = lines.next() {
        // builds the line and returns any errors
        let rule_line = match await_io! {
            build_line(line, &mut lines, line_num, &mut tokenization_data, getter)
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

                
                return Err(e.into_sca_error(rules.lines()))
            }
        };
        rule_lines.push(rule_line);
    }

    // signals to the getter that the rules are done being built
    getter.on_end();

    Ok(AppliableRules {
        lines: rules.lines().collect(),
        rules: rule_lines,
        sources: tokenization_data.take_sources(),
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
    /// Pointers to input (freed on drop)
    ///
    /// Should not be cloned or leaked to other owners
    sources: Vec<*const str>,
}

impl AppliableRules<'_> {
    /// Applies all rules to the input using a runtime, errors are formatted as a string
    #[inline]
    #[io_fn]
    pub fn apply<R: Runtime>(&self, input: &str, runtime: &mut R) -> String {
        await_io! {
            self.apply_fallible(input, runtime)
        }.unwrap_or_else(|e| e.to_string())
    }

    /// Applies all rules to the input using a runtime
    /// 
    /// # Errors
    /// Errors on invalid rules, application that takes too long, and failed io
    #[io_fn]
    pub fn apply_fallible<R: Runtime>(&self, input: &str, runtime: &mut R) -> Result<String, ScaError> {
        let escaped_input = EscapedString::from(input);
        let mut phones = build_phone_list(escaped_input.as_escaped_str());

        let mut line_num = ONE;

        // prepares the runtime for a new set of applications
        runtime.on_start();

        // applies rules
        for rule_line in &self.rules {
            if let Err(e) = await_io! {
                runtime.apply_line(rule_line, &mut phones, line_num)
            } {
                // signals to the runtime that execution is complete
                runtime.on_end();

                return Err(e.into_sca_error(self.lines.iter().copied()));
            }

            line_num = unsafe { NonZero::new_unchecked(line_num.get() + rule_line.lines().get()) };
        }

        // signals to the runtime that execution is complete
        runtime.on_end();

        Ok(phone_list_to_string(&phones))
    }

    /// Extends a rule set with rules from another
    pub fn extend(&mut self, mut other: Self) {
        self.rules.append(&mut other.rules);
        self.lines.append(&mut other.lines);
        // input sources are moved to `self` so it is safe to drop `other`
        self.sources.append(&mut other.sources);
    }
}

impl Drop for AppliableRules<'_> {
    fn drop(&mut self) {
        for source in &self.sources {
            // Safety: Using `AppliableRules` should not
            // leak references to sources and the source
            // pointers should never be cloned
            unsafe {
                source.cast_mut().drop_in_place();
            }
        }
    }
}
