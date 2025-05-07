use crate::{keywords::{ESCAPE_CHAR, SPECIAL_STRS, is_special_char}, sub_string::SubString};

#[cfg(test)]
mod tests;

/// A representation of a phoneme or word boundary
/// 
/// Stores the phoneme's symbol as a reference to the origional text or rules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phone<'s> {
    /// A symbol representing a phoneme
    Symbol(&'s str),
    /// A word boundary
    Bound,
}

impl<'s> Phone<'s> {
    /// Returns the phone's symbol.
    /// If the phone is a boundary, `" "` (space) is returned
    #[must_use]
    pub fn as_str(&self) -> &'s str {
        match self {
            Self::Symbol(symbol) => symbol,
            Self::Bound => " ",
        }
    }

    /// Determines if two phones match
    /// 
    /// Equal phones match and bounds and all-whitespace phones match
    #[must_use]
    pub fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bound, Self::Symbol(symbol)) | (Self::Symbol(symbol), Self::Bound)
                => symbol.chars().all(char::is_whitespace),
            _ => self == other,
        }
    }
}

impl std::fmt::Display for Phone<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Escapes special chars and isolated special strings in input
pub fn escape_input(input: &str) -> String {
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
        // ! (for functionality of other parts of code these should be single characters in length,
        // ! or composed of special characters)
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

    escaped
}

/// Builds a list of phones (as string slices with lifetime 's)
/// from an input (string slice with 's)
/// where each phone is a character or escaped character
/// and reformats whitespace as word bounderies
#[must_use]
pub fn build_phone_list(input: &str) -> Vec<Phone<'_>> {
    let mut substring = SubString::new(input);
    let mut phones = Vec::new();

    for c in input.chars() {
        substring.grow(c);

        match c {
            ESCAPE_CHAR => (),
            '\n' => {
                substring.move_after();
                if !phones.last().is_some_and(|p| p == &Phone::Bound) {
                    phones.push(Phone::Bound);
                }
                phones.push(Phone::Symbol("\n"));
                phones.push(Phone::Bound);
            },
            _ if c.is_whitespace() => {
                substring.move_after();
                if !phones.last().is_some_and(|p| p == &Phone::Bound) {
                    phones.push(Phone::Bound);
                }
            },
            _ => {
                phones.push(Phone::Symbol(substring.take_slice()));
                substring.move_after();
            }
        }
    }

    if !substring.take_slice().is_empty() {
        phones.push(Phone::Symbol(substring.take_slice()));
    }

    phones
}

/// Converts a list of string slices to a string
/// reformating word bounderies as whitespace
#[must_use]
pub fn phone_list_to_string(phone_list: &[Phone]) -> String {
    phone_list
        .iter()
        .fold(String::new(), |acc, phone| acc + phone.as_str())
        .split(&format!("{ESCAPE_CHAR}{ESCAPE_CHAR}"))
        .map(|s| s.replace(ESCAPE_CHAR, ""))
        .reduce(|acc, s| format!("{acc}{ESCAPE_CHAR}{s}"))
        .unwrap_or_default()
        .trim()
        .to_string()
}