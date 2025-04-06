use crate::{tokens::ESCAPE_CHAR, BOUND_STR};

/// A representation of a phoneme
/// 
/// Stores the phoneme's symbol as a reference to the origional text or rules
#[derive(Debug, Clone, Copy)]
pub struct Phone<'s> {
    symbol: &'s str,
}

impl<'s> Phone<'s> {
    /// Creates a new phone with the given symbol
    #[inline]
    pub const fn new(symbol: &'s str) -> Self {
        Self { symbol, }
    }

    /// Creates a word boundery
    #[inline]
    pub const fn new_bound() -> Self {
        Self::new(BOUND_STR)
    }

    /// Gets the symbol of the phone
    #[inline]
    pub const fn symbol(&self) -> &'s str {
        self.symbol
    }
    
    /// Determines if a different phone matches with escaping characters in the first phone removed,
    /// and whitespace treated as bounds
    /// 
    /// **Note**: This is not symetric, a.matches(b) does not imply b.matches(a)
    /// 
    /// ```
    /// use cscsca::{phones::Phone, BOUND_STR};
    /// 
    /// assert!(Phone::new("test").matches(&Phone::new("test")));
    /// assert!(!Phone::new("test").matches(&Phone::new("not test")));
    /// assert!(Phone::new("\\@").matches(&Phone::new("@")));
    /// assert!(!Phone::new("@").matches(&Phone::new("\\@")));
    /// assert!(Phone::new("\\\\@").matches(&Phone::new("\\@")));
    /// assert!(!Phone::new("\\@").matches(&Phone::new("\\@")));
    /// assert!(Phone::new("\\ ").matches(&Phone::new(&format!("{BOUND_STR}"))));
    /// assert!(Phone::new("\\ ").matches(&Phone::new(" ")));
    /// ```
    pub fn matches(&self, other: &Self) -> bool {
        let phone_chars = self.symbol.chars();
        let mut other_chars = other.symbol.chars();

        let mut escape = false;

        for phone_char in phone_chars {
            if phone_char == ESCAPE_CHAR && !escape {
                escape = true;
                continue;
            }

            escape = false;

            if let Some(other_char) = other_chars.next() {
                if phone_char.is_whitespace() && other_char.to_string() == BOUND_STR { continue; }
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
        write!(f, "{}", self.symbol)
    }
}

impl PartialEq for Phone<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol
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
            s
        } else if s.trim().is_empty() {
            BOUND_STR
        } else {
            s
        })
        .map(Phone::new);

    let mut phone_list = Vec::new();

    for phone in phones {
        if phone.symbol() == "\n" {
            phone_list.push(Phone::new_bound());
            phone_list.push(phone);
            phone_list.push(Phone::new_bound());
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
        .replace(&format!("{BOUND_STR}\n{BOUND_STR}"), "\n")
        .replace(BOUND_STR, " ")
        .trim()
        .to_string()
}