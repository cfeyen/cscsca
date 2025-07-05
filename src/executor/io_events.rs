use crate::keywords::{GET_LINE_START, GET_AS_CODE_LINE_START};

/// Events that require IO executed by the `IoGetter` or `Runtime`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoEvent<'s> {
    Runtime(RuntimeIoEvent<'s>),
    Tokenizer(TokenizerIoEvent<'s>),
}

/// IO event that is executed by the `Runtime` during execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeIoEvent<'s> {
    Print { msg: &'s str },
}

/// IO event that is executed by the `IoGetter` when building rules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenizerIoEvent<'s> {
    Get {
        get_type: GetType,
        var: &'s str,
        msg: &'s str,
    },
}

/// How input is interpreted
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GetType {
    /// Input is an escaped phone list
    Phones,
    /// Input is arbitrary code
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