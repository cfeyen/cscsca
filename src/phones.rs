use std::str::from_utf8_unchecked;

use crate::{tokens::ESCAPE_CHAR, BOUND_CHAR};

/// `BOUND_CHAR` as a static str
pub const BOUND_STR: &'static str = unsafe { from_utf8_unchecked(&[BOUND_CHAR as u8]) }; 

/// A representation of a phoneme or word boundary
/// 
/// Stores the phoneme's symbol as a reference to the origional text or rules
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phone<'s> {
    /// A symbol representing a phoneme
    Symbol(&'s str),
    /// A word boundary
    Bound,
}

impl<'s> Phone<'s> {
    /// Creates a new phone that is either a symbol or a bound
    /// depending on the input
    #[inline]
    pub fn new(symbol: &'s str) -> Self {
        if symbol == BOUND_STR {
            Self::Bound
        } else {
            Self::Symbol(symbol)
        }
    }

    /// Returns the phone's symbol.
    /// If the phone is a boundary, `BOUND_STR` is returned
    pub fn as_str(&self) -> &'s str {
        match self {
            Self::Symbol(symbol) => symbol,
            Self::Bound => BOUND_STR,
        }
    }

    /// Determines if a different phone matches with escaping characters in the first phone removed,
    /// and whitespace treated as bounds
    /// 
    /// **Note**: This is not symetric, a.matches(b) does not imply b.matches(a)
    /// 
    /// ```
    /// use cscsca::{phones::{Phone, BOUND_STR}};
    /// 
    /// assert!(Phone::Symbol("test").matches(&Phone::Symbol("test")));
    /// assert!(!Phone::Symbol("test").matches(&Phone::Symbol("not test")));
    /// assert!(Phone::Symbol("\\@").matches(&Phone::Symbol("@")));
    /// assert!(!Phone::Symbol("@").matches(&Phone::Symbol("\\@")));
    /// assert!(Phone::Symbol("\\\\@").matches(&Phone::Symbol("\\@")));
    /// assert!(!Phone::Symbol("\\@").matches(&Phone::Symbol("\\@")));
    /// assert!(Phone::Symbol("\\ ").matches(&Phone::Bound));
    /// assert!(Phone::Symbol("\\ \\ ").matches(&Phone::Bound));
    /// assert!(!Phone::Bound.matches(&Phone::Symbol("\\ ")));
    /// assert!(Phone::Symbol(&format!("\\{BOUND_STR}")).matches(&Phone::Bound));
    /// assert!(!Phone::Bound.matches(&Phone::Symbol(&format!("\\{BOUND_STR}"))));
    /// assert!(Phone::Symbol("\\ ").matches(&Phone::Symbol(" ")));
    /// ```
    pub fn matches(&self, other: &Self) -> bool {
        let symbol = self.as_str();
        let other_symbol = other.as_str();

        let phone_chars = symbol.chars();
        let mut other_chars = other_symbol.chars();

        let mut escape = false;
        let mut in_whitespace = false;

        for phone_char in phone_chars {
            // removes an escape character ('\')
            // and marks an immeadiately following one not to be escaped
            if phone_char == ESCAPE_CHAR && !escape {
                escape = true;
                continue;
            }

            escape = false;

            // phone and the previous are whitespace skip to the next phone
            if in_whitespace && phone_char.is_whitespace() {
                continue;
            }

            if let Some(other_char) = other_chars.next() {
                // marks the loop as in whitespace if the character is whitespace
                // and the other character is a `BOUND_STR`
                if phone_char.is_whitespace() {
                    // if the other phone is a bound str or whitespace,
                    // the loop is marked as in whitespace and moved to the next iteration,
                    // otherwise, false is returned
                    if other_char.to_string() == BOUND_STR || other_char.is_whitespace() {
                        in_whitespace = true;
                        continue;
                    } else {
                        return false
                    }
                } else {
                    in_whitespace = false;
                }

                if phone_char != other_char { return false; }
            } else {
                return false;
            }
        }

        other_chars.next().is_none()
    }
}

impl std::fmt::Display for Phone<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Builds a list of phones (as string slices with lifetime 's)
/// from an input (string slice with 's)
/// and reformats whitespace as word bounderies
pub fn build_phone_list(input: &str) -> Vec<Phone<'_>> {
    let phones = input
        .split("")
        .filter(|s| !s.is_empty())
        .map(|s| if s == "\n" {
            Phone::Symbol(s)
        } else if let BOUND_STR | "" = s.trim() {
            Phone::Bound
        } else {
            Phone::Symbol(s)
        });

    let mut phone_list = Vec::new();

    for phone in phones {
        if phone == Phone::Symbol("\n") {
            phone_list.push(Phone::Bound);
            phone_list.push(phone);
            phone_list.push(Phone::Bound);
        } else {
            phone_list.push(phone);
        }
    }

    phone_list
}

/// Converts a list of string slices to a string
/// reformating word bounderies as whitespace
pub fn phone_list_to_string(phone_list: &[Phone]) -> String {
    phone_list
        .iter()
        .fold(String::new(), |acc, phone| format!("{acc}{phone}"))
        .replace(&format!("{BOUND_CHAR}\n{BOUND_CHAR}"), "\n")
        .replace(BOUND_CHAR, " ")
        .trim()
        .to_string()
}