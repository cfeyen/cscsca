use std::{fmt::Display, rc::Rc};

use crate::{
    ir::tokens::IrToken,
    keywords::ARG_SEP_CHAR,
    phones::Phone,
    tokens::ScopeType
};

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
        match self {
            Self::Any { id: None } => write!(f, "{}", IrToken::Any),
            Self::Any { id: Some(id) } => write!(f, "{id}{}", IrToken::Any),
            Self::Gap { id: None } => write!(f, "{}", IrToken::Gap),
            Self::Gap { id: Some(name) } => write!(f, "{} {}", IrToken::Label(name), IrToken::Gap),
            Self::Phone(phone) => write!(f, "{phone}"),
            Self::OptionalScope { id: None, content } => {
                let s = content
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ");

                write!(f, "{} {s} {}", ScopeType::Optional.fmt_start(), ScopeType::Optional.fmt_end())
            }
            Self::OptionalScope { id: Some(id), content } => {
                write!(f, "{}{}", id, Self::OptionalScope { id: None, content: content.clone() })
            }
            Self::SelectionScope { id: None, options } => {
                let s = options
                    .iter()
                    .map(|list| {
                        list
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .collect::<Vec<_>>()
                    .join(&format!("{ARG_SEP_CHAR} "));

                write!(f, "{} {s} {}", ScopeType::Selection.fmt_start(), ScopeType::Selection.fmt_end())
            }
            Self::SelectionScope { id: Some(id), options } => {
                write!(f, "{}{}", id, Self::SelectionScope { id: None, options: options.clone() })
            }
        }
    }
}

/// A scope label
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeId<'s> {
    /// Named label
    Name(&'s str),
    /// The number that a scope is of its type in an input or output
    /// Along with the scope type and the id of the parent scope
    /// (allows for scope matching inference)
    IOUnlabeled {
        id_num: usize,
        label_type: LabelType,
        parent: Option<Rc<Self>>,
    }
}

impl Display for ScopeId<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IOUnlabeled { id_num: _, label_type: _, parent: _ } => Ok(()),
            Self::Name(name) => write!(f, "{}", IrToken::Label(name)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LabelType {
    Scope(ScopeType),
    Any,
}

impl Display for LabelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scope(kind) => write!(f, "{kind}"),
            Self::Any => write!(f, "{}", RuleToken::Any { id: None }),
        }
    }
}