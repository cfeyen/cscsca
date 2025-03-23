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
    pub fn new(symbol: &'s str) -> Self {
        Self { symbol, }
    }

    /// Creates a word boundery
    pub fn new_bound() -> Self {
        Self::new(BOUND_STR)
    }

    /// Gets the symbol of the phone
    pub fn symbol(&self) -> &'s str {
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