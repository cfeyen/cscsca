use std::fmt::Display;

use crate::{
    keywords::{ANY_CHAR, ARG_SEP_CHAR, COND_CHAR, GAP_STR, LABEL_PREFIX}, phones::Phone, rules::conditions::{AndType, CondType}, tokens::{ScopeType, Shift}
};

/// Tokens that make up the intermediate representation of sound shifts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrToken<'s> {
    /// A phone literal
    Phone(Phone<'s>),
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
    /// The main focus and type of a condition or anti-condition
    CondType(CondType),
    /// The start of a scope
    ScopeStart(ScopeType),
    /// The end of a scope
    ScopeEnd(ScopeType),
}

impl Display for IrToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Any => format!("{ANY_CHAR}"),
            Self::ArgSep => format!("{ARG_SEP_CHAR}"),
            Self::Break(r#break) => format!("{break}"),
            Self::Gap => GAP_STR.to_string(),
            Self::CondType(focus) => format!("{focus}"),
            Self::Phone(phone) => phone.to_string(),
            Self::ScopeEnd(kind) => kind.fmt_end().to_string(),
            Self::ScopeStart(kind) => kind.fmt_start().to_string(),
            Self::Label(name) => format!("{LABEL_PREFIX}{name}"),
        };

        write!(f, "{s}")
    }
}

/// Breaks in phone lists
/// 
/// Seperate input, output, conditions and anti-conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Break {
    /// Seperates input and output
    /// 
    /// Denotes how a change should be applied
    Shift(Shift),
    /// Starts a contition
    Cond,
    /// Starts an anti-condition
    AntiCond,
    /// A union between conditions where both must succeed
    /// or the first must succeed and the next must fail
    And(AndType),
}

impl Display for Break {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Shift(shift) => format!("{shift}"),
            Self::Cond => format!("{COND_CHAR}"),
            Self::AntiCond => format!("{COND_CHAR}{COND_CHAR}"),
            Self::And(and_type) => format!("{and_type}"),
        };

        write!(f, "{s}")
    }
}