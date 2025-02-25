use crate::BOUND_STR;

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