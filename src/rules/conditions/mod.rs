use std::fmt::Display;

use crate::{
    keywords::{INPUT_PATTERN_STR, MATCH_CHAR, AND_CHAR, NOT_CHAR},
    matcher::{tokens_match_phones_from_left, tokens_match_phones_from_right, Choices, MatchError},
    tokens::Direction,
    phones::Phone
};

use super::tokens::RuleToken;

#[cfg(test)]
mod tests;

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

    /// Sets the additional required condition
    #[inline]
    pub fn set_and(&mut self, and_type: AndType, and: Self) {
        self.and = Some((and_type, Box::new(and)));
    }

    /// Checks if the condition matches the phones in a list around a given index
    /// assuming the input of a given size matches and based on the application direction
    pub fn eval<'c, 'r>(&'r self, phones: &[Phone<'s>], phone_index: usize, input_len: usize, choices: &mut Choices<'c, 'r, 's>, dir: Direction) -> Result<bool, MatchError<'r, 's>> {
        let cond_succeeds = match self.kind {
            CondType::Pattern => {
                let (before_phones, after_phones) = match dir {
                    Direction::Ltr => (&phones[0..phone_index], phones.get(phone_index + input_len..).unwrap_or_default()),
                    Direction::Rtl => {
                        let before_phones = if input_len <= phone_index {
                            &phones[0..=phone_index - input_len]
                        } else {
                            &[]
                        };

                        (before_phones, phones.get(phone_index + 1..).unwrap_or_default())
                    },
                };

                let before_matches = tokens_match_phones_from_right(&self.before, before_phones, choices)?;
                let after_matches = tokens_match_phones_from_left(&self.after, after_phones, choices)?;

                before_matches && after_matches
            },
            CondType::Match => {
                let mut left = Vec::new();

                for token in &self.before {
                    if let RuleToken::Phone(phone) = token {
                        left.push(*phone);
                    } else {
                        return Err(MatchError::LeftMustBePhones(token));
                    }
                }

                tokens_match_phones_from_left(&self.after, &left, choices)?
            }
        };

        Ok(cond_succeeds && if let Some((and_type, and_cond)) = &self.and {
            let cond_res = and_cond.eval(phones, phone_index, input_len, choices, dir)?;

            match and_type {
                AndType::And => cond_res,
                AndType::AndNot => !cond_res,
            }
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