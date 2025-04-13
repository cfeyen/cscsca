use std::fmt::Display;

pub const LTR_CHAR: char = '>';
pub const RTL_CHAR: char = '<';
pub const OPTIONAL_START_CHAR: char = '(';
pub const OPTIONAL_END_CHAR: char = ')';
pub const SELECTION_START_CHAR: char = '{';
pub const SELECTION_END_CHAR: char = '}';

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shift {
    pub dir: Direction,
    pub kind: ShiftType,
}

impl Display for Shift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self { dir, kind: ShiftType::Stay } => format!("{dir}"),
            Self { dir, kind: ShiftType::Move } => format!("{dir}{dir}"),
        };

        write!(f, "{s}")
    }
}

/// The direction a sound change applies to the word in
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    /// Left-to-Right
    Ltr,
    /// Right-to-Left
    Rtl,
}

impl Direction {
    /// Changes n by dist according to the direction (wraps instead of overflowing)
    pub fn change_by(self, n: usize, dist: usize) -> usize {
        match self {
            Self::Ltr => n.wrapping_add(dist),
            Self::Rtl => n.wrapping_sub(dist),
        }
    }

    /// Changes n by 1 according to the direction (wraps instead of overflowing)
    pub fn change_by_one(self, n: usize) -> usize {
        self.change_by(n, 1)
    }

    /// Returns the first index required for traversing a list according to direction
    /// 
    /// (LTR returns 0, RTL returns list length - 1)
    pub fn start_index<T>(self, list: &[T]) -> usize {
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
}

impl ScopeType {
    pub fn fmt_start(self) -> char {
        match self {
            ScopeType::Optional => OPTIONAL_START_CHAR,
            ScopeType::Selection => SELECTION_START_CHAR,
        }
    }

    pub fn fmt_end(self) -> char {
        match self {
            ScopeType::Optional => OPTIONAL_END_CHAR,
            ScopeType::Selection => SELECTION_END_CHAR,
        }
    }
}

impl Display for ScopeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}...{}", self.fmt_start(), self.fmt_end())
    }
}