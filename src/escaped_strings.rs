use crate::keywords::{is_special_char, ESCAPE_CHAR, SPECIAL_STRS};

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
    /// ## Warning
    /// If the `EscapedStr` is dropped a memory leak may occur
    pub fn leak<'a>(self) -> EscapedStr<'a> {
        EscapedStr(self.0.leak())
    }

    /// Returns a reference the internal `String` as an `EscapedStr`
    pub fn inner(&self) -> EscapedStr<'_> {
        EscapedStr(&self.0)
    }
}

impl From<&str> for EscapedString {
    fn from(value: &str) -> Self {
        escape_input(value)
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
fn escape_input(input: &str) -> EscapedString {
    let mut escaped = String::new();
    let mut chars = input.chars();

    let mut i = 0;
    let mut last_is_whitespace = false;

    'outer: while let Some(c) = chars.next() {
        // Gets the substring from c onward
        let c_and_after = input.get(i..).unwrap_or_default();

        // handles special characters
        if is_special_char(c) {
            escaped.push(ESCAPE_CHAR);
            escaped.push(c);
            last_is_whitespace = false;
            i += c.len_utf8();
            continue 'outer;
        }

        // handles isolated strings
        if last_is_whitespace {
            for s in SPECIAL_STRS {
                if c_and_after.starts_with(s) && {
                    let after_s = c_and_after.get(s.len()..);
                    after_s.is_none_or(|s| s.is_empty() || s.starts_with(char::is_whitespace))
                } {
                    escaped.push(c);

                    for _ in s.chars() {
                        chars.next();
                    }
    
                    i += s.len();
                    
                    escaped.push(ESCAPE_CHAR);
                    escaped += s;
                    last_is_whitespace = false;
                    continue 'outer;
                }
            }
        }
        
        // handles normal characters
        last_is_whitespace = c.is_whitespace();
        i += c.len_utf8();
        escaped.push(c);
    }

    EscapedString(escaped)
}