use std::sync::Arc;
use std::fmt::Display;

use crate::{applier::matcher::{tokens_match_phones_from_left, tokens_match_phones_from_right, Choices, MatchError}, meta_tokens::{Direction, ScopeType, Shift}, phones::Phone, tokens::ir::{Break, IrToken}};

/// A collection of data that define a sound change rule
#[derive(Debug, Clone, PartialEq)]
pub struct SoundChangeRule<'s> {
    pub kind: Shift,
    /// The tokens that represent an input
    pub input: Vec<RuleToken<'s>>,
    /// The tokens that represent what should replace the input
    pub output: Vec<RuleToken<'s>>,
    /// The tokens that represent the enviroment in which the input should be replaced
    pub conds: Vec<Cond<'s>>,
    /// The tokens that represent the enviroment in which the input should not be replaced
    pub anti_conds: Vec<Cond<'s>>,
}

impl Display for SoundChangeRule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let input = self.input
            .iter()
            .map(|t| format!("{t}"))
            .collect::<Vec<_>>()
            .join(" ");

        let output = self.output
            .iter()
            .map(|t| format!("{t}"))
            .collect::<Vec<_>>()
            .join(" ");

        let mut conds = String::new();
        for cond in &self.conds {
            conds += &format!(" {} {cond}", IrToken::Break(Break::Cond));
        }

        let mut anti_conds = String::new();
        for anti_cond in &self.anti_conds {
            anti_conds += &format!(" {} {anti_cond}", IrToken::Break(Break::AntiCond));
        }
        
        write!(f, "{} {} {}{}{}", input, &self.kind, output, conds, anti_conds)
    }
}

/// A set of tokens that represent the enviroment before and after the input pattern
#[derive(Debug, Clone, PartialEq)]
pub enum Cond<'s> {
    Match {
        before: Vec<RuleToken<'s>>,
        after: Vec<RuleToken<'s>>,
    }
}

impl<'s> Cond<'s> {
    /// Checks if the condition matches the phones in a list around a given index
    /// assuming the input of a given size matches and based on the application direction
    pub fn eval<'a>(&'a self, phones: &[Phone<'s>], phone_index: usize, input_len: usize, choices: &mut Choices<'a, 's>, dir: Direction) -> Result<bool, MatchError<'a, 's>> {
        match self {
            Self::Match { before, after } => {
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

                let before_matches = tokens_match_phones_from_right(before, before_phones, choices)?;
                let after_matches = tokens_match_phones_from_left(after, after_phones, choices)?;

                Ok(before_matches && after_matches)
            }
        }
    }

    #[inline]
    pub const fn new_match(before: Vec<RuleToken<'s>>, after: Vec<RuleToken<'s>>) -> Self {
        Self::Match { before, after }
    }
}

impl Display for Cond<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Match { before, after } => {
                let before = before
                    .iter()
                    .map(|t| format!("{t}"))
                    .collect::<Vec<_>>()
                    .join(" ");

                let after = after
                    .iter()
                    .map(|t| format!("{t}"))
                    .collect::<Vec<_>>()
                    .join(" ");

                write!(f, "{} {} {}", before, IrToken::Input, after)
            }
        }
    }
}

impl Default for Cond<'_> {
    #[inline]
    fn default() -> Self {
        Self::Match {
            before: Default::default(),
            after: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuleToken<'s> {
    /// A phone literal
    Phone(Phone<'s>),
    /// A scope that chooses between several phone lists
    SelectionScope {
        id: Option<ScopeId<'s>>,
        options: Vec<Vec<Self>>,
    },
    /// A scope that either does or doesn't insert a phone list
    OptionalScope {
        id: Option<ScopeId<'s>>,
        content: Vec<Self>,
    },
    /// Any non bound phone, can agree with others of the same label
    Any { id: Option<ScopeId<'s>>, },
    /// A lazily evaluated gap of any non-negative length that does not contatin a word bound
    Gap { id: Option<&'s str>, }
}

impl Display for RuleToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Any { id: None } => {
                format!("{}", IrToken::Any)
            },
            Self::Any { id: Some(id) } => {
                format!("{id}{}", IrToken::Any)
            },
            Self::Gap { id: None } => format!("{}", IrToken::Gap),
            Self::Gap { id: Some(name) } => format!("{} {}", IrToken::Label(name), IrToken::Gap),
            Self::Phone(phone) => phone.to_string(),
            Self::OptionalScope { id: None, content } => {
                let s = content
                    .iter()
                    .map(|t| format!("{t}"))
                    .collect::<Vec<_>>()
                    .join(" ");

                format!("{} {s} {}", ScopeType::Optional.fmt_start(), ScopeType::Optional.fmt_end())
            }
            Self::OptionalScope { id: Some(id), content } => {
                format!("{}{}", id, Self::OptionalScope { id: None, content: content.to_vec() })
            }
            Self::SelectionScope { id: None, options } => {
                let s = options
                    .iter()
                    .map(|list| {
                        list
                            .iter()
                            .map(|t| format!("{t}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .collect::<Vec<_>>()
                    .join(&format!("{} ", IrToken::ArgSep));

                format!("{} {s} {}", ScopeType::Selection.fmt_start(), ScopeType::Selection.fmt_end())
            }
            Self::SelectionScope { id: Some(id), options } => {
                format!("{}{}", id, Self::SelectionScope { id: None, options: options.to_vec() })
            }
        };

        write!(f, "{}", s)
    }
}

/// A scope label
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeId<'s> {  // 'a is the lifetime of the soucre code, 'b is the lifetime of a scope reference
    /// Named label
    Name(&'s str),
    /// The number that a scope is of its type in an input or output
    /// Along with the scope type and the id of the parent scope
    /// (allows for scope matching inference)
    IOUnlabeled {
        id_num: usize,
        label_type: LabelType,
        parent: Option<Arc<Self>>,
    }
}

impl Display for ScopeId<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::IOUnlabeled { id_num: _, label_type: _, parent: _ } => {
                String::new()
            },
            Self::Name(name) => { format!("{}", IrToken::Label(name)) }
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LabelType {
    Scope(ScopeType),
    Any,
}

impl Display for LabelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Scope(kind) => format!("{kind}"),
            Self::Any => format!("{}", RuleToken::Any { id: None }),
        };

        write!(f, "{}", s)
    }
}