use crate::keywords::{DEFINITION_PREFIX, LABEL_PREFIX, VARIABLE_PREFIX};

/// Prefixes added to literal strings to modify their effects
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Prefix {
    /// Inserts a predefined value in its place
    Definition,
    /// Allows for scopes to agree in what they add to the phone list
    Label,
    /// Inserts a value declared at runtime
    Variable
}

impl Prefix {
    /// Returns the character associated with the `Prefix`
    pub const fn char(self) -> char {
        match self {
            Self::Definition => DEFINITION_PREFIX,
            Self::Label => LABEL_PREFIX,
            Self::Variable => VARIABLE_PREFIX,
        }
    }
}

impl std::fmt::Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.char())
    }
}