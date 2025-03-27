pub const DEFINITION_PREFIX: char = '@';
pub const SELECTION_PREFIX: char = '$';
pub const VARIABLE_PREFIX: char = '%';

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

impl std::fmt::Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::Definition => DEFINITION_PREFIX,
            Self::Label => SELECTION_PREFIX,
            Self::Variable => VARIABLE_PREFIX
        };

        write!(f, "{}", c)
    }
}