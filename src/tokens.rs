use crate::{ir::tokens::IrToken, keywords::{AND_CHAR, ANY_CHAR, REPETITION_END_CHAR, REPETITION_START_CHAR, INPUT_PATTERN_STR, LTR_CHAR, MATCH_CHAR, NOT_CHAR, OPTIONAL_END_CHAR, OPTIONAL_START_CHAR, RTL_CHAR, SELECTION_END_CHAR, SELECTION_START_CHAR}};

use std::{fmt::Display, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shift {
    pub dir: Direction,
    pub kind: ShiftType,
}

impl Display for Shift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self { dir, kind: ShiftType::Stay } => write!(f, "{dir}"),
            Self { dir, kind: ShiftType::Move } => write!(f, "{dir}{dir}"),
        }
    }
}

/// The direction a sound change applies to the word in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Left-to-Right
    Ltr,
    /// Right-to-Left
    Rtl,
}

impl Direction {
    /// Changes n by dist according to the direction (wraps instead of overflowing)
    pub const fn change_by(self, n: usize, dist: usize) -> usize {
        match self {
            Self::Ltr => n.wrapping_add(dist),
            Self::Rtl => n.wrapping_sub(dist),
        }
    }

    /// Changes n by 1 according to the direction (wraps instead of overflowing)
    pub const fn change_by_one(self, n: usize) -> usize {
        self.change_by(n, 1)
    }

    /// Returns the first index required for traversing a list according to direction
    /// 
    /// (LTR returns 0, RTL returns list length - 1)
    pub const fn start_index<T>(self, list: &[T]) -> usize {
        match self {
            Self::Ltr => 0,
            Self::Rtl => list.len().wrapping_sub(1),
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::Ltr => LTR_CHAR,
            Self::Rtl => RTL_CHAR,
        };

        write!(f, "{c}")
    }
}

/// The type of shift
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftType {
    /// The next phones analyzed are before/after the newly inserted phones
    Move,
    /// The next phones analyzed are the newly inserted phones
    Stay,
}

/// The type of a scope or scope bound
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeType {
    /// A scope that hold one item and either adds it to the phone list or doesn't
    Optional,
    /// A scope that adds one of its items to the phone list
    Selection,
    /// A scope that represents a repetition of phones
    Repetition,
}

impl ScopeType {
    pub const fn fmt_start(self) -> char {
        match self {
            ScopeType::Optional => OPTIONAL_START_CHAR,
            ScopeType::Selection => SELECTION_START_CHAR,
            ScopeType::Repetition => REPETITION_START_CHAR,
        }
    }

    pub const fn fmt_end(self) -> char {
        match self {
            ScopeType::Optional => OPTIONAL_END_CHAR,
            ScopeType::Selection => SELECTION_END_CHAR,
            ScopeType::Repetition => REPETITION_END_CHAR,
        }
    }
}

impl Display for ScopeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}...{}", self.fmt_start(), self.fmt_end())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AndType {
    #[default]
    And,
    AndNot
}

impl std::fmt::Display for AndType {
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
    /// A deliminator for a match between two groups of tokens
    Match,
}

impl std::fmt::Display for CondType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pattern => write!(f, "{INPUT_PATTERN_STR}"),
            Self::Match => write!(f, "{MATCH_CHAR}"),
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

impl std::fmt::Display for ScopeId<'_> {
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

impl std::fmt::Display for LabelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scope(kind) => write!(f, "{kind}"),
            Self::Any => write!(f, "{ANY_CHAR}"),
        }
    }
}