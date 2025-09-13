use std::fmt::Display;

use crate::keywords::{AND_CHAR, INPUT_PATTERN_STR, MATCH_CHAR, NOT_CHAR};

use super::tokens::RuleToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AndType {
    #[default]
    And,
    AndNot
}

impl Display for AndType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::And => write!(f, "{AND_CHAR}"),
            Self::AndNot => write!(f, "{AND_CHAR}{NOT_CHAR}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CondType {
    /// The input in a pattern based condition or anti-condition
    #[default]
    Pattern,
    /// A deliminator for a match between to groups of tokens
    Match,
}

impl Display for CondType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pattern => write!(f, "{INPUT_PATTERN_STR}"),
            Self::Match => write!(f, "{MATCH_CHAR}"),
        }
    }
}

/// A pair of token lists can be compared based on the kind of the condition
/// either to the enviroment around a phone or to each other
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cond<'s> {
    kind: CondType,
    before: Vec<RuleToken<'s>>,
    after: Vec<RuleToken<'s>>,
    and: Option<(AndType, Box<Self>)>,
}

impl<'s> Cond<'s> {
    #[inline]
    pub const fn new(kind: CondType, before: Vec<RuleToken<'s>>, after: Vec<RuleToken<'s>>) -> Self {
        Self { kind, before, after, and: None }
    }

    /// Appends a new and or and not clause
    pub fn add_and(&mut self, and_type: AndType, and: Self) {
        if let Some((_, and_clause)) = &mut self.and {
            and_clause.add_and(and_type, and);
        } else {
            self.and = Some((and_type, Box::new(and)));
        }
    }

    /// gets the type of condition
    #[inline]
    pub const fn kind(&self) -> CondType {
        self.kind
    }

    /// gets the left side of the condition
    #[inline]
    pub fn left(&self) -> &[RuleToken<'s>] {
        &self.before
    }

    /// gets the right side of the condition
    #[inline]
    pub fn right(&self) -> &[RuleToken<'s>] {
        &self.after
    }

    /// gets the clause condition
    #[inline]
    pub fn and(&self) -> Option<(AndType, &Self)> {
        self.and.as_ref()
            .map(|(and_type, and_cond)| (*and_type, and_cond.as_ref()))
    }
}

impl Display for Cond<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let before = self.before
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");

        let after = self.after
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");

        write!(f, "{before} {} {after}", self.kind)
    }
}