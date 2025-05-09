#![warn(clippy::correctness)]
#![warn(clippy::suspicious)]
#![warn(clippy::complexity)]
#![warn(clippy::perf)]
#![warn(clippy::pedantic)]
#![warn(clippy::panic)]
#![warn(clippy::style)]

use std::error::Error;

use runtime::Runtime;

pub(crate) mod ir;
pub(crate) mod phones;
pub(crate) mod tokens;
pub(crate) mod rules;
pub(crate) mod applier;
pub(crate) mod matcher;
pub(crate) mod sub_string;
pub(crate) mod escaped_strings;
pub mod runtime;
pub mod keywords;

#[cfg(test)]
mod tests;

#[cfg(feature = "gen_vscode_grammar")] // | other | other | ...
pub mod tooling_gen;

/// Applies sca source code to an input string
/// 
/// Returns a string of either the final text or a formatted error
#[inline]
#[must_use]
pub fn apply(input: &str, code: &str) -> String {
    apply_with_runtime(input, code, &Runtime::default())
}

/// Applies sca source code to an input string
/// 
/// ## Errors
/// Errors are the result of providing invalid code, failed io, or application timing out
#[inline]
pub fn apply_fallible(input: &str, code: &str) -> Result<String, ScaError> {
    apply_fallible_with_runtime(input, code, &Runtime::default())
}

/// Applies sca source code to an input string
/// 
/// Returns a string of either the final text or a formatted error
#[inline]
#[must_use]
pub fn apply_with_runtime(input: &str, code: &str, runtime: &Runtime) -> String {
    apply_fallible_with_runtime(input, code, runtime)
        .unwrap_or_else(|e| e.to_string())
}

/// Applies sca source code to an input string,
/// 
/// ## Errors
/// Errors are the result of providing invalid code, failed io, or application timing out
#[inline]
pub fn apply_fallible_with_runtime(input: &str, code: &str, runtime: &Runtime) -> Result<String, ScaError> {
    runtime.apply(input, code)
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct ScaError(String);

impl Error for ScaError {}

impl ScaError {
    /// Builds a new `ScaError` from any error,
    /// with the line and line number it occurred on
    fn from_error<E: Error + ?Sized>(e: &E, line: &str, line_num: usize) -> Self {
        Self(format!("{}Error:{} {e}\nLine {line_num}: {line}", ansi::RED, ansi::RESET))
    }
}

impl std::fmt::Display for ScaError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// ANSI color codes
#[cfg(not(feature = "no_ansi"))]
pub mod ansi {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[93m";
    pub const GREEN: &str = "\x1b[92m";
    pub const BLUE: &str = "\x1b[94m";
    pub const MAGENTA: &str = "\x1b[35m";
}

#[cfg(feature = "no_ansi")]
mod ansi {
    pub const RESET: &str = "";
    pub const RED: &str = "";
    pub const BLUE: &str = "";
}