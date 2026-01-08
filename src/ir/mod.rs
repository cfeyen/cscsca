use std::num::NonZero;

use crate::{
    ONE, executor::io_events::{GetType, IoEvent, RuntimeIoEvent, TokenizerIoEvent}, ir::tokenization_data::TokenizationData, keywords::{DEFINITION_LINE_START, DEFINITION_PREFIX, ESCAPE_CHAR, INPUT_PATTERN_STR, VARIABLE_PREFIX}, lexer::{Sir, sir::SirToken}, phones::Phone, tokens::CondType
};

use tokens::IrToken;
use prefix::Prefix;

pub mod tokens;
pub mod prefix;
pub mod tokenization_data;

#[cfg(test)]
mod tests;

/// Takes an expression or statement from a SIR iterator and creates an `IrLine`
/// or produces an error with the number of lines the line takes up
pub fn ir_line_from_sir<'s>(sir: &mut Sir<'s>, tokenization_data: &mut TokenizationData<'s>, lazy_expansions: &mut Vec<&'s str>) -> Result<IrLine<'s>, (IrError<'s>, NonZero<usize>)> {
    let mut lines = ONE;
    let line = get_expr(sir);

    // handles empty lines
    if line.is_empty() {
        return Ok(IrLine::Empty { lines });
    }

    // handles statements
    match &line[0] {
        SirToken::DefinitionDeclaration(_) => {
            let mut sir_iter = line[1..].iter()
                .skip_while(|t| matches!(t, SirToken::Whitespace(_)));
        
            if let Some(name) = sir_iter.next() && let SirToken::Phone(name) = name {
                let name = name.str();

                let sir_tokens = sir_iter.copied().collect::<Vec<_>>();
                for t in sir_tokens.iter() {
                    if let SirToken::NonPhoneEscape('\n', _) | SirToken::EndOfExpr(_) = t {
                        lines = unsafe { NonZero::new_unchecked(lines.get() + 1) }
                    }
                }
                let ir_line = ir_line_from_sir(&mut Sir::new(sir_tokens), tokenization_data, lazy_expansions);

                let content = match ir_line? {
                        IrLine::Empty { .. }  => Vec::new(),
                        IrLine::IoEvent(_) => return Err((IrError::StatementParseError, lines)),
                        IrLine::Ir { tokens, .. } => tokens,
            
                };

                tokenization_data.set_definition(name, content);

                Ok(IrLine::Empty { lines })
            } else {
                Err((IrError::UnnamedDefinition, lines))
            }
        }
        SirToken::LazyDefinitionDeclaration(_) => {
            let mut sir_iter = line[1..].iter()
                .skip_while(|t| matches!(t, SirToken::Whitespace(_)));
        
            if let Some(name) = sir_iter.next() && let SirToken::Phone(name) = name {
                let name = name.str();
                let sir_tokens = sir_iter.copied().collect::<Vec<_>>();
                for t in sir_tokens.iter() {
                    if let SirToken::NonPhoneEscape('\n', _) | SirToken::EndOfExpr(_) = t {
                        lines = unsafe { NonZero::new_unchecked(lines.get() + 1) }
                    }
                }
                tokenization_data.set_lazy_definition(name, Sir::new(sir_tokens));

                Ok(IrLine::Empty { lines })
            } else {
                Err((IrError::UnnamedDefinition, lines))
            }
        }
        SirToken::GetAsCodeCommand(_) => {
            let mut sir_iter = line[1..].iter()
                .skip_while(|t| matches!(t, SirToken::Whitespace(_)));

            if let Some(SirToken::Phone(var)) = sir_iter.next() {
                Ok(IrLine::IoEvent(IoEvent::Tokenizer(TokenizerIoEvent::Get {
                    get_type: GetType::Code,
                    var: var.str(),
                    msg: sir_iter.next()
                        .map(|t| if let SirToken::Message(msg, _) = t { msg } else { "" })
                        .unwrap_or_default()
                })))
            } else {
                Err((IrError::InvalidGetFormat(GetType::Code), lines))
            }
        },
        SirToken::GetCommand(_) => {
            let mut sir_iter = line[1..].iter()
                .skip_while(|t| matches!(t, SirToken::Whitespace(_)));

            if let Some(SirToken::Phone(var)) = sir_iter.next() {
                Ok(IrLine::IoEvent(IoEvent::Tokenizer(TokenizerIoEvent::Get {
                    get_type: GetType::Phones,
                    var: var.str(),
                    msg: sir_iter.next()
                        .map(|t| if let SirToken::Message(msg, _) = t { msg } else { "" })
                        .unwrap_or_default()
                })))
            } else {
                Err((IrError::InvalidGetFormat(GetType::Phones), lines))
            }
        },
        SirToken::PrintCommand(_) => {
            let mut sir_iter = line[1..].iter()
                .skip_while(|t| matches!(t, SirToken::Whitespace(_)));

            let msg = sir_iter.next()
                .map(|t| if let SirToken::Message(msg, _) = t { msg } else { "" })
                .unwrap_or_default();

            Ok(IrLine::IoEvent(IoEvent::Runtime(RuntimeIoEvent::Print { msg })))
        },
        // handles non-statement lines
        _ => {
            let (ir_res, lines) = sir_expr_to_ir_line(line, tokenization_data, lazy_expansions);
            let ir = match ir_res {
                Err(e) => return Err((e, lines)),
                Ok(ir) => ir,
            };

            if ir.is_empty() {
                Ok(IrLine::Empty { lines })
            } else {
                Ok(IrLine::Ir { tokens: ir, lines })
            }
        }
    }
}

/// Converts a HIR expression to an `IrLine` or `IrError` with the number of lines the rule occurs on
pub fn sir_expr_to_ir_line<'s>(sir: Vec<SirToken<'s>>, tokenization_data: &mut TokenizationData<'s>, lazy_expansions: &mut Vec<&'s str>) -> (Result<Vec<IrToken<'s>>, IrError<'s>>, NonZero<usize>) {
    let mut ir = Vec::new();
    let mut lines = ONE;

    for token in sir {
        ir.push(match token {
            SirToken::Any(_) => IrToken::Any,
            SirToken::ArgSep(_) => IrToken::ArgSep,
            SirToken::Bound(_) => IrToken::Phone(Phone::Bound),
            SirToken::Break(b, _) => IrToken::Break(b),
            SirToken::Comment(_) | SirToken::Whitespace(_) => continue,
            SirToken::CondFocus(ct, _) => IrToken::CondType(ct),
            SirToken::Definition(def) => {
                if let Err(e) = tokenization_data.get_definition(def.str(), &mut ir, lazy_expansions) {
                    return (Err(e), lines);
                }

                continue;
            }
            SirToken::InvalidPrefix(prefix, _) => return (Err(IrError::EmptyPrefix(prefix)), lines),
            SirToken::Label(label) => IrToken::Label(label.str()),
            SirToken::Negative(_) => IrToken::Negative,
            SirToken::EndOfExpr(_) | SirToken::NonPhoneEscape('\n', _) => {
                lines = unsafe { NonZero::new_unchecked(lines.get() + 1) };
                continue;
            }
            SirToken::NonPhoneEscape(c, _) => return (Err(IrError::BadEscape(Some(c))), lines),
            SirToken::Phone(symbol) => IrToken::Phone(Phone::Symbol(symbol.str())),
            SirToken::ScopeEnd(st, _) => IrToken::ScopeEnd(st),
            SirToken::ScopeStart(st, _) => IrToken::ScopeStart(st),
            SirToken::SpecialStr(s) => match s.str() {
                INPUT_PATTERN_STR => IrToken::CondType(CondType::Pattern),
                s => return (Err(IrError::InvalidPhone(s)), lines)
            },
            SirToken::Variable(var) => {
                let content = tokenization_data.get_variable(var.str());

                let tokens = match content {
                    Err(e) => return (Err(e), lines),
                    Ok(tokens) => tokens,
                };

                for token in tokens {
                    ir.push(*token);
                }

                continue;
            },
            SirToken::DefinitionDeclaration(_) | SirToken::LazyDefinitionDeclaration(_)
            | SirToken::GetCommand(_) | SirToken::GetAsCodeCommand(_)
            | SirToken::PrintCommand(_) | SirToken::Message(_, _) => return (Err(IrError::StatementParseError), lines)
        });
    }

    (Ok(ir), lines)
}

/// Gets the tokens for an expression or statement from a HIR iterator
fn get_expr<'s>(sir: &mut Sir<'s>) -> Vec<SirToken<'s>> {
    let mut line = Vec::new();

    while let Some(token) = sir.next() {
        if matches!(token, SirToken::EndOfExpr(_)) {
            break;
        }

        line.push(token);
    }

    line
}

/// A list of `IrTokens`, a command, or nothing representing a line of source code
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum IrLine<'s> {
    Ir {
        tokens: Vec<IrToken<'s>>,
        lines: NonZero<usize>,
    },
    IoEvent(IoEvent<'s>),
    Empty { lines: NonZero<usize> },
}

impl IrLine<'_> {
    /// Gets the number of lines an `IrLine` takes up
    pub const fn lines(&self) -> NonZero<usize> {
        match self {
            Self::Ir {lines, .. } => *lines,
            _ => ONE,
        }
    }
}

/// Errors that occur when parsing raw text to tokens
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum IrError<'s> {
    RecursiveLazyDefiniton(&'s str),
    EmptyPrefix(Prefix),
    UndefinedDefinition(&'s str),
    UndefinedVariable(&'s str),
    UnnamedDefinition,
    BadEscape(Option<char>),
    InvalidGetFormat(GetType),
    InvalidPhone(&'s str),
    StatementParseError,
}

impl std::error::Error for IrError<'_> {}

impl std::fmt::Display for IrError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecursiveLazyDefiniton(name) => write!(f, "Lazy definition '{DEFINITION_PREFIX}{name}' is recursive"),
            Self::EmptyPrefix(prefix) => write!(f, "Found prefix '{prefix}' without a following identifier"),
            Self::UndefinedDefinition(name) => write!(f, "Undefined definiton '{DEFINITION_PREFIX}{name}'"),
            Self::UndefinedVariable(name) => write!(f, "Undefined variable '{VARIABLE_PREFIX}{name}'"),
            Self::UnnamedDefinition => write!(f, "Found '{DEFINITION_LINE_START}' with out a following name"),
            Self::BadEscape(None) => write!(f, "Found '{ESCAPE_CHAR}' with no following character"),
            Self::BadEscape(Some(c)) => write!(f, "Escaped normal character '{}' ({ESCAPE_CHAR}{c})", c.escape_debug()),
            Self::InvalidGetFormat(get_type) => write!(f, "Invalid format after '{get_type}', expected variable name and message"),
            Self::InvalidPhone(s) => write!(f, "'{s}' is not a valid phone, label, or name"),
            Self::StatementParseError => write!(f, "Found invalid statement"),
        }
    }
}
 