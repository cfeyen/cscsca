use std::fmt::Display;

use crate::{
    keywords::{ANY_CHAR, ARG_SEP_CHAR, COND_CHAR, LABEL_PREFIX, NOT_CHAR},
    phones::Phone,
    tokens::{AndType, CondType, ScopeType, Shift}
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
    /// The main focus and type of a condition or anti-condition
    CondType(CondType),
    /// The start of a scope
    ScopeStart(ScopeType),
    /// The end of a scope
    ScopeEnd(ScopeType),
    // gap negator
    Negative,
}

impl Display for IrToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => write!(f, "{ANY_CHAR}"),
            Self::ArgSep => write!(f, "{ARG_SEP_CHAR}"),
            Self::Break(r#break) => write!(f, "{break}"),
            Self::CondType(focus) => write!(f, "{focus}"),
            Self::Phone(phone) => write!(f, "{phone}"),
            Self::ScopeEnd(kind) => write!(f, "{}", kind.fmt_end()),
            Self::ScopeStart(kind) => write!(f, "{}", kind.fmt_start()),
            Self::Label(name) => write!(f, "{LABEL_PREFIX}{name}"),
            Self::Negative => write!(f, "{NOT_CHAR}"),
        }
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
        match self {
            Self::Shift(shift) => write!(f, "{shift}"),
            Self::Cond => write!(f, "{COND_CHAR}"),
            Self::AntiCond => write!(f, "{COND_CHAR}{COND_CHAR}"),
            Self::And(and_type) => write!(f, "{and_type}"),
        }
    }
}