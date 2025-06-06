use crate::keywords::{GET_LINE_START, GET_AS_CODE_LINE_START};

/// Non rule commands executed by the runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command<'s> {
    RuntimeCommand(RuntimeCommand<'s>),
    ComptimeCommand(ComptimeCommand<'s>),
}

/// Non rule commands executed by the runtime durring tokenization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCommand<'s> {
    Print { msg: &'s str },
}

/// Non rule commands executed by the runtime durring rule execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComptimeCommand<'s> {
    Get {
        get_type: GetType,
        var: &'s str,
        msg: &'s str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GetType {
    Phones,
    Code,
}

impl std::fmt::Display for GetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Phones => write!(f, "{GET_LINE_START}"),
            Self::Code => write!(f, "{GET_AS_CODE_LINE_START}"),
        }
    }
}