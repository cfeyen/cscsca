#[cfg(test)]
mod tests;

use crate::{
    escaped_strings::EscapedString,
    executor::{
        compile_line,
        runtime::{Runtime, RuntimeApplier},
        getter::IoGetter,
    },
    ir::tokenization_data::TokenizationData,
    phones::{build_phone_list, phone_list_to_string},
    rules::RuleLine,
    ScaError,
};

/// Compiles all rules to a form that may be applied more easily
/// 
/// ## Errors
/// Errors on invalid rules or failed io
pub fn compile_rules<'s, G: IoGetter>(rules: &'s str, getter: &mut G) -> Result<AppliableRules<'s>, ScaError> {
    let mut line_num = 0;
    let mut rule_lines = Vec::new();
    let mut tokenization_data = TokenizationData::new();

    for line in rules.lines() {
        line_num += 1;

        let rule_line = match compile_line(line, line_num, &mut tokenization_data, getter) {
            Ok(rule_line) => rule_line,
            Err(e) => {
                drop(rule_lines);
                // Safety: Since the output is a ScaError,
                // which owns all of its values, and `rule_lines` is dropped,
                // no references remain to the sources buffer in `tokenization_data`
                unsafe { tokenization_data.free_sources() };
                return Err(e)
            }
        };
        rule_lines.push(rule_line);
    }

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
    /// The compiled rules
    rules: Vec<RuleLine<'s>>,
    /// Pointers to input (freed when dropped)
    sources: Vec<*const str>,
}

impl AppliableRules<'_> {
    /// Applies all rules to the input using a runtime, errors are formatted as a string
    #[inline]
    pub fn apply<R: Runtime>(&self, input: &str, runtime: &mut R) -> String {
        self.apply_fallible(input, runtime)
            .unwrap_or_else(|e| e.to_string())
    }

    /// Applies all rules to the input using a runtime
    /// 
    /// ## Errors
    /// Errors on invalid rules, application that takes too long, and failed io
    pub fn apply_fallible<R: Runtime>(&self, input: &str, runtime: &mut R) -> Result<String, ScaError> {
        let escaped_input = EscapedString::from(input);
        let mut phones = build_phone_list(escaped_input.as_escaped_str());

        let mut line_num = 0;

        for rule_line in &self.rules {
            let line = self.lines.get(line_num).copied().unwrap_or_default();
            line_num += 1;

            runtime.apply_line(rule_line, &mut phones, line, line_num)?;
        }

        Ok(phone_list_to_string(&phones))
    }
}

impl Drop for AppliableRules<'_> {
    fn drop(&mut self) {
        for source in &self.sources {
            // Safety: Using `CompiledRules` should not
            // leak references to sources and the source
            // pointers should never be cloned
            unsafe {
                source.cast_mut().drop_in_place();
            }
        }
    }
}
