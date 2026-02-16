use crate::{keywords::{is_isolated_char, is_special_char, ESCAPE_CHAR}};

/// A `String` that has all special characters escaped
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct EscapedString(String);

impl EscapedString {
    /// Shrinks the internal `String` to have no extra capacity 
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    /// Leaks into an aribitrary-lifetime `EscapedStr`
    /// 
    /// # Warning
    /// If the `EscapedStr` is dropped a memory leak may occur
    pub fn leak<'a>(self) -> EscapedStr<'a> {
        EscapedStr(self.0.leak())
    }

    /// Returns a reference the internal `String` as an `EscapedStr`
    pub fn as_escaped_str(&self) -> EscapedStr<'_> {
        EscapedStr(&self.0)
    }
}

impl From<&str> for EscapedString {
    fn from(value: &str) -> Self {
        Self(escape_input(value))
    }
}

/// A `&str` that has all special characters escaped
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EscapedStr<'a>(&'a str);

impl<'a> EscapedStr<'a> {
    /// Returns the internal `&str`
    pub fn inner(&self) -> &'a str {
        self.0
    }
}

/// Escapes special chars and isolated special strings in input
fn escape_input(input: &str) -> String {
    let mut escaped = String::new();

    for c in input.chars() {
        if is_special_char(c) || is_isolated_char(c) {
            escaped.push(ESCAPE_CHAR);
        }

        escaped.push(c);
    }

    escaped
}