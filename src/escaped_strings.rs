use crate::{ir::IrError, keywords::{is_isolated_char, is_isolation_bound, is_special_char, ESCAPE_CHAR, SPECIAL_STRS}};

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

/// Ensures all escapes are valid
pub fn check_escapes(input: &str) -> Result<(), IrError<'_>> {
    let mut chars = input.chars();
    let mut i = 0;
    let mut last_is_whitespace_or_always_special_char = true;

    'outer: while let Some(c) = chars.next() {
        i += c.len_utf8();

        if c == ESCAPE_CHAR {
            if let Some(next) = chars.next() {
                if is_special_char(next) {
                    i += next.len_utf8();
                    last_is_whitespace_or_always_special_char = true;
                    continue;
                }

                if last_is_whitespace_or_always_special_char && is_isolated_char(c) && {
                    let after_next: &str = input.get(i+next.len_utf8()..).unwrap_or_default();
                    after_next.is_empty() || after_next.starts_with(is_isolation_bound)
                } {
                    i += next.len_utf8();
                    last_is_whitespace_or_always_special_char = false;
                    continue;
                }

                let after_c = &input[i..];
                i += next.len_utf8();

                if last_is_whitespace_or_always_special_char {
                    for s in SPECIAL_STRS {
                        if after_c.starts_with(s) && {
                            let after_s = &after_c.get(s.len()..).unwrap_or_default();
                            after_s.is_empty() || after_s.starts_with(is_isolation_bound)
                        } {
                            for c in s.chars().skip(1) {
                                i += c.len_utf8();
                            }
                            continue 'outer;
                        }
                    }
                }

                return Err(IrError::BadEscape(Some(next)))
            }

            return Err(IrError::BadEscape(None))
        }

        last_is_whitespace_or_always_special_char = c.is_whitespace();
    }

    Ok(())
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

#[cfg(test)]
#[test]
fn niche_escapes() {
    assert_eq!("\\_\\/".to_string(), escape_input("_/"));
    assert_eq!("\\_a".to_string(), escape_input("_a"));

    // isolated only escapes
    assert!(check_escapes("\\_a").is_err());
    assert!(check_escapes("\\_ a").is_ok());
}