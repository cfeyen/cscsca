use std::fmt::Display;

use crate::{applier::matcher::{tokens_match_phones_from_left, tokens_match_phones_from_right, Choices, MatchError}, meta_tokens::Direction, phones::Phone};

use super::sound_change_rule::RuleToken;

pub const INPUT_STR: &str = "_";
pub const EQUALITY_CHAR: char = '=';

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CondType {
    /// The input in a condition or anti-condition
    MatchInput,
    /// A deliminator for a match between to groups of tokens
    Equality,
}

impl Display for CondType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MatchInput => write!(f, "{INPUT_STR}"),
            Self::Equality => write!(f, "{EQUALITY_CHAR}"),
        }
    }
}

/// A set of tokens that represent the enviroment before and after the input pattern
#[derive(Debug, Clone, PartialEq)]
pub struct Cond<'s> {
    kind: CondType,
    before: Vec<RuleToken<'s>>,
    after: Vec<RuleToken<'s>>,
}

impl<'s> Cond<'s> {
    #[inline]
    pub const fn new(kind: CondType, before: Vec<RuleToken<'s>>, after: Vec<RuleToken<'s>>) -> Self {
        Self { kind, before, after }
    }

    #[inline]
    pub const fn kind(&self) -> CondType {
        self.kind
    }

    #[inline]
    pub fn before(&self) -> &[RuleToken<'s>] {
        &self.before
    }

    #[inline]
    pub fn after(&self) -> &[RuleToken<'s>] {
        &self.after
    }

    /// Checks if the condition matches the phones in a list around a given index
    /// assuming the input of a given size matches and based on the application direction
    pub fn eval<'a>(&'a self, phones: &[Phone<'s>], phone_index: usize, input_len: usize, choices: &mut Choices<'a, 's>, dir: Direction) -> Result<bool, MatchError<'a, 's>> {
        match self.kind {
            CondType::MatchInput => {
                let (before_phones, after_phones) = match dir {
                    Direction::LTR => (&phones[0..phone_index], &phones[phone_index + input_len..]),
                    Direction::RTL => {
                        let before_phones = if input_len <= phone_index {
                            &phones[0..=phone_index - input_len]
                        } else {
                            &[]
                        };

                        (before_phones, &phones[phone_index + 1..])
                    },
                };

                let before_matches = tokens_match_phones_from_right(&self.before, before_phones, choices)?;
                let after_matches = tokens_match_phones_from_left(&self.after, after_phones, choices)?;

                Ok(before_matches && after_matches)
            },
            CondType::Equality => {
                Ok(self.before == self.after)
            }
        }
    }
}

impl Default for Cond<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            kind: CondType::MatchInput,
            before: Default::default(),
            after: Default::default(),
        }
    }
}

impl Display for Cond<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let before = self.before
            .iter()
            .map(|t| format!("{t}"))
            .collect::<Vec<_>>()
            .join(" ");

        let after = self.after
            .iter()
            .map(|t| format!("{t}"))
            .collect::<Vec<_>>()
            .join(" ");

        write!(f, "{before} {} {after}", self.kind)
    }
}