use crate::{
    ir::{prefix::Prefix, tokens::Break},
    lexer::token_types::{PhoneValidStr, Span},
    tokens::{CondType, ScopeType}
};

/// A Scoped Intermediate Representation token
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SirToken<'s> {
    /// A phone
    Phone(PhoneValidStr<'s>),
    /// A definition insertion
    Definition(PhoneValidStr<'s>),
    /// A label
    Label(PhoneValidStr<'s>),
    /// A variable insertion
    Variable(PhoneValidStr<'s>),
    /// A prefix that does not start a name
    InvalidPrefix(Prefix, Span),
    /// A string with special effects
    SpecialStr(PhoneValidStr<'s>),
    /// An escaped character that does not result in part of a phone or name
    NonPhoneEscape(char, Span),
    /// A shift or condition segmenting token
    Break(Break, Span),
    /// A condition focus
    CondFocus(CondType, Span),
    /// The start of a scope
    ScopeStart(ScopeType, Span),
    /// The end of a scope
    ScopeEnd(ScopeType, Span),
    /// Any non-boundary phone
    Any(Span),
    /// A scope argument seperator
    ArgSep(Span),
    /// A word boundary character
    Bound(Span),
    /// The start of a repetition exclusion
    Negative(Span),
    /// The start of a definition declaration
    DefinitionDeclaration(Span),
    /// The start of a lazy definition declaration
    LazyDefinitionDeclaration(Span),
    /// The start of a get statement
    GetCommand(Span),
    /// The start of a get as code statement
    GetAsCodeCommand(Span),
    /// The start of a print statement
    PrintCommand(Span),
    /// A comment
    Comment(Span),
    /// A printable message
    Message(&'s str, Span),
    /// Whitespace
    Whitespace(Span),
    /// The end of an unescaped line
    EndOfExpr(Span),
}

#[cfg(feature = "debug_tokens")]
impl SirToken<'_> {
    /// Gets the location data for a token
    pub fn span(&self) -> &Span {
        match self {
            Self::Phone(s) | Self::Definition(s)
            | Self::Label(s) | Self::Variable(s)
            | Self::SpecialStr(s) => s.span(),
            Self::InvalidPrefix(_, s) | Self::NonPhoneEscape(_, s)
            | Self::Break(_, s) | Self::CondFocus(_, s)
            | Self::ScopeStart(_, s) | Self::ScopeEnd(_, s)
            | Self::Negative(s) | Self::Any(s)
            | Self::ArgSep(s) | Self::Bound(s)
            | Self::DefinitionDeclaration(s) | Self::LazyDefinitionDeclaration(s)
            | Self::GetCommand(s) | Self::GetAsCodeCommand(s)
            | Self::PrintCommand(s) | Self::Comment(s)
            | Self::Whitespace(s) | Self::EndOfExpr(s)
            | Self::Message(_, s) => s
        }
    }
}

