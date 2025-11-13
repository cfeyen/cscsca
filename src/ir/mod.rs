use std::num::NonZero;

use crate::{
    executor::io_events::{GetType, IoEvent},
    keywords::{DEFINITION_LINE_START, DEFINITION_PREFIX, ESCAPE_CHAR, VARIABLE_PREFIX},
    ONE,
};

use tokens::IrToken;
use prefix::Prefix;

pub mod tokens;
pub mod prefix;
pub mod tokenization_data;
pub mod tokenizer;

#[cfg(test)]
mod tests;

/// A list of `IrTokens`, a command, or nothing representing a line of source code
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum IrLine<'s> {
    Ir {
        tokens: Vec<IrToken<'s>>,
        lines: NonZero<usize>,
    },
    IoEvent(IoEvent<'s>),
    Empty,
}

impl IrLine<'_> {
    pub const fn lines(&self) -> NonZero<usize> {
        match self {
            Self::Ir {lines, .. } => *lines,
            _ => ONE,
        }
    }
}

/// Errors that occur when parsing raw text to tokens
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum IrError<'s> {
    RecursiveLazyDefiniton(&'s str),
    EmptyPrefix(Prefix),
    UndefinedDefinition(&'s str),
    UndefinedVariable(&'s str),
    EmptyDefinition,
    BadEscape(Option<char>),
    ReservedCharacter(char),
    InvalidGetFormat(GetType),
}

impl std::error::Error for IrError<'_> {}

impl std::fmt::Display for IrError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecursiveLazyDefiniton(name) => write!(f, "Lazy definition '{DEFINITION_PREFIX}{name}' is recursive"),
            Self::EmptyPrefix(prefix) => write!(f, "Found prefix '{prefix}' without a following identifier"),
            Self::UndefinedDefinition(name) => write!(f, "Undefined definiton '{DEFINITION_PREFIX}{name}'"),
            Self::UndefinedVariable(name) => write!(f, "Undefined variable '{VARIABLE_PREFIX}{name}'"),
            Self::EmptyDefinition => write!(f, "Found '{DEFINITION_LINE_START}' with out a following name"),
            Self::BadEscape(None) => write!(f, "Found '{ESCAPE_CHAR}' with no following character"),
            Self::BadEscape(Some(c)) => write!(f, "Escaped normal character '{c}' ({ESCAPE_CHAR}{c})"),
            Self::ReservedCharacter(c) => write!(f, "Found reserved character '{c}' consider escaping it ('{ESCAPE_CHAR}{c}')"),
            Self::InvalidGetFormat(get_type) => write!(f, "Invalid format after '{get_type}', expected variable name and message"),
        }
    }
}
 