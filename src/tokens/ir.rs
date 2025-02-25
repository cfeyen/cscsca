use crate::{tokens::prefix::SELECTION_PREFIX, meta_tokens::{Shift, ScopeType}};

pub const ANY_CHAR: char = '*';
pub const ARG_SEP_CHAR: char = ',';
pub const COND_CHAR: char = '/';
pub const GAP_STR: &str = "..";
pub const INPUT_STR: &str = "_";

/// Tokens that make up the intermediate representation of sound shifts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IrToken<'s> {
    /// A phone literal
    Phone(&'s str),
    /// The identifier of a scope's selection
    Label(&'s str),
    /// A break between phone lists
    Break(Break),
    /// Any non bound phone
    Any,
    /// An item seperator for selection scopes
    ArgSep,
    /// A gap of size 0 or greater that does not contain a word boundery
    Gap,
    /// The input in a condition or anti-condition
    Input,
    /// The start of a scope
    ScopeStart(ScopeType),
    /// The end of a scope
    ScopeEnd(ScopeType),
}

impl std::fmt::Display for IrToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Any => format!("{ANY_CHAR}"),
            Self::ArgSep => format!("{ARG_SEP_CHAR}"),
            Self::Break(r#break) => format!("{}", r#break),
            Self::Gap => GAP_STR.to_string(),
            Self::Input => INPUT_STR.to_string(),
            Self::Phone(phone) => phone.to_string(),
            Self::ScopeEnd(kind) => kind.fmt_end().to_string(),
            Self::ScopeStart(kind) => kind.fmt_start().to_string(),
            Self::Label(name) => format!("{SELECTION_PREFIX}{name}"),
        };

        write!(f, "{}", s)
    }
}

/// Breaks in phone lists
/// 
/// Seperate input, output, conditions and anti_conditions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Break {
    /// Seperates input and output
    /// 
    /// Denotes how a change should be applied
    Shift(Shift),
    /// Starts a contition
    Cond,
    /// Starts an anti-condition
    AntiCond,
}

impl std::fmt::Display for Break {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Shift(shift) => format!("{shift}"),
            Self::Cond => format!("{COND_CHAR}"),
            Self::AntiCond => format!("{COND_CHAR}{COND_CHAR}"),
        };

        write!(f, "{}", s)
    }
}