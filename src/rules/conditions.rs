use std::fmt::Display;

use crate::{applier::matcher::{tokens_match_phones_from_left, tokens_match_phones_from_right, Choices, MatchError}, meta_tokens::Direction, phones::Phone};

use super::sound_change_rule::RuleToken;

pub const INPUT_STR: &str = "_";
pub const EQUALITY_CHAR: char = '=';

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CondType {
    /// The input in a condition or anti-condition
    #[default]
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

/// A pair of token lists can be compared based on the kind of the condition
/// either to the enviroment around a phone or to each other
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cond<'s> {
    kind: CondType,
    before: Vec<RuleToken<'s>>,
    after: Vec<RuleToken<'s>>,
    and: Option<Box<Self>>,
}

impl<'s> Cond<'s> {
    #[inline]
    pub const fn new(kind: CondType, before: Vec<RuleToken<'s>>, after: Vec<RuleToken<'s>>) -> Self {
        Self { kind, before, after, and: None }
    }

    /// Sets the additional required condition
    #[inline]
    pub fn set_and(&mut self, and: Self) {
        self.and = Some(Box::new(and));
    }

    /// Checks if the condition matches the phones in a list around a given index
    /// assuming the input of a given size matches and based on the application direction
    pub fn eval<'a>(&'a self, phones: &[Phone<'s>], phone_index: usize, input_len: usize, choices: &mut Choices<'a, 's>, dir: Direction) -> Result<bool, MatchError<'a, 's>> {
        let cond_succeeds = match self.kind {
            CondType::MatchInput => {
                let (before_phones, after_phones) = match dir {
                    Direction::Ltr => (&phones[0..phone_index], &phones[phone_index + input_len..]),
                    Direction::Rtl => {
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

                before_matches && after_matches
            },
            CondType::Equality => {
                let accumulate_display = |acc, token| format!("{acc}{token}");
                self.before.iter().fold(String::new(), accumulate_display) ==
                self.after.iter().fold(String::new(), accumulate_display)
            }
        };

        Ok(cond_succeeds && if let Some(and) = &self.and {
            and.eval(phones, phone_index, input_len, choices, dir)?
        } else {
            true
        })
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