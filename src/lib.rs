use runtime::Runtime;
use commands::PrintLog;

pub(crate) mod tokens;
pub mod phones;
pub(crate) mod meta_tokens;
pub(crate) mod rules;
pub(crate) mod applier;
pub(crate) mod commands;
pub mod runtime;

#[cfg(test)]
mod tests;

pub const BOUND_CHAR: char = '#';

/// Applies sca source code to an input string
/// 
/// Returns a string of either the final text or a formatted error and the print log
#[inline]
pub fn apply(input: &str, code: &str) -> (String, PrintLog) {
    apply_with_runtime(input, code, &Runtime::default())
}

/// Applies sca source code to an input string, logging prints
/// 
/// Returns a result of either the final text or a formatted error
#[inline]
pub fn apply_fallible(input: &str, code: &str) -> (Result<String, String>, PrintLog) {
    apply_fallible_with_runtime(input, code, &Runtime::default())
}

/// Applies sca source code to an input string
/// 
/// Returns a string of either the final text or a formatted error and the print log
pub fn apply_with_runtime(input: &str, code: &str, runtime: &Runtime) -> (String, PrintLog) {
    let (result, log) = apply_fallible_with_runtime(input, code, runtime);

    (result.unwrap_or_else(|e| e), log)
}

/// Applies sca source code to an input string, logging prints
/// 
/// Returns a result of either the final text or a formatted error
#[inline]
pub fn apply_fallible_with_runtime(input: &str, code: &str, runtime: &Runtime) -> (Result<String, String>, PrintLog) {
    runtime.apply(input, code)
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