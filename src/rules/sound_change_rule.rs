use std::sync::Arc;
use std::fmt::{Display, Write as _};

use crate::{meta_tokens::{ScopeType, Shift}, phones::Phone, tokens::ir::{Break, IrToken}};

use super::conditions::Cond;

/// A collection of data that define a sound change rule
#[derive(Debug, Clone, PartialEq, Eq)]
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
            _ = write!(conds, " {} {cond}", IrToken::Break(Break::Cond));
        }

        let mut anti_conds = String::new();
        for anti_cond in &self.anti_conds {
            _ = write!(anti_conds, " {} {anti_cond}", IrToken::Break(Break::AntiCond));
        }
        
        write!(f, "{} {} {}{}{}", input, &self.kind, output, conds, anti_conds)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
                format!("{}{}", id, Self::OptionalScope { id: None, content: content.clone() })
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
                format!("{}{}", id, Self::SelectionScope { id: None, options: options.clone() })
            }
        };

        write!(f, "{s}")
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

        write!(f, "{s}")
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

        write!(f, "{s}")
    }
}