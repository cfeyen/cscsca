use crate::{
    escaped_strings::check_escapes,
    keywords::{is_special_char, AND_CHAR, ANY_CHAR, ARG_SEP_CHAR, BOUND_CHAR, COMMENT_LINE_START, COND_CHAR, DEFINITION_LINE_START, DEFINITION_PREFIX, ESCAPE_CHAR, GAP_STR, GET_AS_CODE_LINE_START, GET_LINE_START, INPUT_PATTERN_STR, LABEL_PREFIX, LTR_CHAR, MATCH_CHAR, NOT_CHAR, OPTIONAL_END_CHAR, OPTIONAL_START_CHAR, PRINT_LINE_START, RTL_CHAR, SELECTION_END_CHAR, SELECTION_START_CHAR, SPECIAL_STRS, VARIABLE_PREFIX}, phones::Phone, rules::conditions::{AndType, CondType},
    executor::commands::{Command, ComptimeCommand, GetType, RuntimeCommand},
    sub_string::SubString,
    tokens::{Direction, ScopeType, Shift, ShiftType},
};

use tokenization_data::TokenizationData;
use tokens::{Break, IrToken};
use prefix::Prefix;

pub mod tokens;
pub mod prefix;
pub mod tokenization_data;

#[cfg(test)]
mod tests;

/// A list of `IrTokens`, a command, or nothing representing a line of source code
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum IrLine<'s> {
    Ir(Vec<IrToken<'s>>),
    Cmd(Command<'s>),
    Empty,
}

/// Converts source code into intermediate representation tokens
pub fn tokenize_line_or_create_command<'s>(line: &'s str, tokenization_data: &mut TokenizationData<'s>) -> Result<IrLine<'s>, IrError<'s>> {
    Ok(if let Some(definition_content) = line.strip_prefix(DEFINITION_LINE_START) {
        // handles definitions
        let ir = tokenize_line(definition_content, tokenization_data)?;

        if let Some(IrToken::Phone(name)) = ir.first() {
            tokenization_data.set_definition(name.as_str(), ir[1..].into());
            IrLine::Empty
        } else {
            return Err(IrError::EmptyDefinition);
        }
    } else if line.starts_with(COMMENT_LINE_START) {
        // handles comments
        IrLine::Empty
    } else if let Some(args) = line.strip_prefix(PRINT_LINE_START) {
        // handles print statement
        IrLine::Cmd(Command::RuntimeCommand(RuntimeCommand::Print { msg: args.trim() }))
    } else if let Some(args) = line.strip_prefix(GET_AS_CODE_LINE_START) {
        // handles get as code
        if let Some((var, msg)) = args.trim().split_once(char::is_whitespace) {
            IrLine::Cmd(Command::BuildtimeCommand(ComptimeCommand::Get {
                get_type: GetType::Code,
                var,
                msg: msg.trim(),
            }))
        } else {
            return Err(IrError::InvalidGetFormat(GetType::Code))
        }
    } else if let Some(args) = line.strip_prefix(GET_LINE_START) {
        // handles get
        if let Some((var, msg)) = args.trim().split_once(char::is_whitespace) {
            IrLine::Cmd(Command::BuildtimeCommand(ComptimeCommand::Get {
                get_type: GetType::Phones,
                var,
                msg: msg.trim(),
            }))
        } else {
            return Err(IrError::InvalidGetFormat(GetType::Code))
        }
    } else {
        // handles rules
        let mut ir_line = IrLine::Ir(tokenize_line(line, tokenization_data)?);
        
        // converts empty rules to the empty varient
        if let IrLine::Ir(ir) = &ir_line {
            if ir.is_empty() { ir_line = IrLine::Empty; }
        }

        ir_line
    })
}

/// Converts a line to tokens
fn tokenize_line<'s>(line: &'s str, tokenization_data: &TokenizationData<'s>) -> Result<Vec<IrToken<'s>>, IrError<'s>> {
    let chars = line.chars();
    let mut tokens = Vec::new();
    let mut prefix = None;
    let mut slice = SubString::new(line);
    let mut escape = false;

    for c in chars {
        match c {
            // handles escapes
            _ if escape => {
                slice.grow(c);
                escape = false;
            },
            ESCAPE_CHAR => {
                slice.grow(c);
                escape = true;
            },
            // handles prefixes
            DEFINITION_PREFIX => start_prefix(Prefix::Definition, &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            LABEL_PREFIX => start_prefix(Prefix::Label, &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            VARIABLE_PREFIX => start_prefix(Prefix::Variable, &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            // handles scope bounds
            OPTIONAL_START_CHAR => push_phone_and(IrToken::ScopeStart(ScopeType::Optional), &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            OPTIONAL_END_CHAR => push_phone_and(IrToken::ScopeEnd(ScopeType::Optional), &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            SELECTION_START_CHAR => push_phone_and(IrToken::ScopeStart(ScopeType::Selection), &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            SELECTION_END_CHAR => push_phone_and(IrToken::ScopeEnd(ScopeType::Selection), &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            // handles simple one-to-one char to token pushes
            AND_CHAR => push_phone_and(IrToken::Break(Break::And(AndType::And)), &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            NOT_CHAR => {
                let token = match tokens.last() {
                    Some(IrToken::Break(Break::And(AndType::And))) => {
                        tokens.pop();
                        IrToken::Break(Break::And(AndType::AndNot))
                    },
                    Some(IrToken::Break(Break::Cond)) => {
                        tokens.pop();
                        IrToken::Break(Break::AntiCond)
                    },
                    _ => return Err(IrError::UnexpectedNot),
                };

                push_phone_and(token, &mut tokens, &mut slice, &mut prefix, tokenization_data)?;
            }
            ANY_CHAR => push_phone_and(IrToken::Any, &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            ARG_SEP_CHAR => push_phone_and(IrToken::ArgSep, &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            BOUND_CHAR => push_phone_and(IrToken::Phone(Phone::Bound), &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            MATCH_CHAR => push_phone_and(IrToken::CondType(CondType::Match), &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            // handles compound char to token pushes
            LTR_CHAR => {
                let kind = if let Some(IrToken::Break(Break::Shift(Shift { dir: Direction::Ltr, kind: ShiftType::Stay }))) = tokens.last() {
                    tokens.pop();
                    ShiftType::Move
                } else {
                    ShiftType::Stay
                };

                push_phone_and(IrToken::Break(Break::Shift(Shift { dir: Direction::Ltr, kind })), &mut tokens, &mut slice, &mut prefix, tokenization_data)?;
            },
            RTL_CHAR => {
                let kind = if let Some(IrToken::Break(Break::Shift(Shift { dir: Direction::Rtl, kind: ShiftType::Stay }))) = tokens.last() {
                    tokens.pop();
                    ShiftType::Move
                } else {
                    ShiftType::Stay
                };

                push_phone_and(IrToken::Break(Break::Shift(Shift { dir: Direction::Rtl, kind })), &mut tokens, &mut slice, &mut prefix, tokenization_data)?;
            },
            COND_CHAR => {
                let cond_type = if let Some(IrToken::Break(Break::Cond)) = tokens.last() {
                    tokens.pop();
                    Break::AntiCond
                } else {
                    Break::Cond
                };

                push_phone_and(IrToken::Break(cond_type), &mut tokens, &mut slice, &mut prefix, tokenization_data)?;
            },
            // whitespace
            _ if c.is_whitespace() => {
                push_phone(&mut tokens, &mut slice, &mut prefix, tokenization_data)?;
                slice.skip(c);
            },
            // other chars
            _ => slice.grow(c)
        }
    }

    push_phone(&mut tokens, &mut slice, &mut prefix, tokenization_data)?;

    Ok(tokens)
}


/// Pushes the slice according to `push_phone`, then sets the prefix
/// and moves the slice over an extra character to account for the prefix character
fn start_prefix<'s>(new_prefix: Prefix, tokens: &mut Vec<IrToken<'s>>, slice: &mut SubString<'s>, prefix: &mut Option<Prefix>, tokenization_data: &TokenizationData<'s>) -> Result<(), IrError<'s>> {
    match prefix {
        Some(prefix) if slice.is_empty() => {
            Err(IrError::EmptyPrefix(*prefix))
        }
        _ => {
            push_phone(tokens, slice, prefix, tokenization_data)?;
            slice.skip_byte();

            *prefix = Some(new_prefix);
            Ok(())
        }
    }
}

/// Pushes the slice according to `push_phone`, then pushes the provided token
/// and moves the slice over an extra character to account for that token
fn push_phone_and<'s>(token: IrToken<'s>, tokens: &mut Vec<IrToken<'s>>, slice: &mut SubString<'s>, prefix: &mut Option<Prefix>, tokenization_data: &TokenizationData<'s>) -> Result<(), IrError<'s>> {
    push_phone(tokens, slice, prefix, tokenization_data)?;
    slice.skip_byte();
    tokens.push(token);
    Ok(())
}

/// Pushes the slice as a phone and prepares it to start the next slice
/// 
/// Handles escape validity and input pattern and gap generation
/// 
/// If there is a prefix, it either expands the phone as a definition or
/// inserts a selection token and resets the prefix to None
/// 
/// If the slice is empty nothing is pushed (returns an error if there is a prefix)
/// 
/// If the slice is the input pattern and there is no prefix,
/// an input token is pushed instead of a phone
fn push_phone<'s>(tokens: &mut Vec<IrToken<'s>>, slice: &mut SubString<'s>, prefix: &mut Option<Prefix>, tokenization_data: &TokenizationData<'s>) -> Result<(), IrError<'s>> {
    let literal = slice.take_slice();
    slice.move_after();

    check_escapes(literal)?;
    check_reserved(literal)?;

    match (&prefix, literal) {
        (None, INPUT_PATTERN_STR) => tokens.push(IrToken::CondType(CondType::Pattern)),
        (None, GAP_STR) => tokens.push(IrToken::Gap),
        (None, "") => (),
        (None, _) => tokens.push(IrToken::Phone(Phone::Symbol(literal))),
        (Some(Prefix::Definition), _) => {
            let content = tokenization_data.get_definition(literal)?;

            for token in content {
                tokens.push(*token);
            }
        },
        (Some(Prefix::Label), _) => tokens.push(IrToken::Label(literal)),
        (Some(Prefix::Variable), _) => {
            let content = tokenization_data.get_variable(literal)?;

            for token in content {
                tokens.push(*token);
            }
        },
    }

    *prefix = None;
    Ok(())
}

/// Ensures all special characters are escaped
fn check_reserved(input: &str) -> Result<(), IrError<'_>> {
    if SPECIAL_STRS.contains(&input) {
        return Ok(());
    }

    let mut chars = input.chars();

    let mut escaped = false;

    while let Some(c) = chars.next() {
        match c {
            ESCAPE_CHAR if !escaped => escaped = true,
            _ if is_special_char(c) && !escaped
                => return Err(IrError::ReservedCharacter(c)),
            _ => escaped = false,
        }
    }

    Ok(())
}

/// Errors that occur when parsing raw text to tokens
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum IrError<'s> {
    EmptyPrefix(Prefix),
    UndefinedDefinition(&'s str),
    UndefinedVariable(&'s str),
    EmptyDefinition,
    BadEscape(Option<char>),
    ReservedCharacter(char),
    UnexpectedNot,
    InvalidGetFormat(GetType),
}

impl std::error::Error for IrError<'_> {}

impl std::fmt::Display for IrError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPrefix(prefix) => write!(f, "Found prefix '{prefix}' without a following identifier"),
            Self::UndefinedDefinition(name) => write!(f, "Undefined definiton '{DEFINITION_PREFIX}{name}'"),
            Self::UndefinedVariable(name) => write!(f, "Undefined definiton '{VARIABLE_PREFIX}{name}'"),
            Self::EmptyDefinition => write!(f, "Found '{DEFINITION_LINE_START}' with out a following name"),
            Self::BadEscape(None) => write!(f, "Found '{ESCAPE_CHAR}' with no following character"),
            Self::BadEscape(Some(c)) => write!(f, "Escaped normal character '{c}' ({ESCAPE_CHAR}{c})"),
            Self::ReservedCharacter(c) => write!(f, "Found reserved character '{c}' consider escaping it ('{ESCAPE_CHAR}{c}')"),
            Self::UnexpectedNot => write!(f, "Found '{NOT_CHAR}' not after '{COND_CHAR}' or '{AND_CHAR}'"),
            Self::InvalidGetFormat(get_type) => write!(f, "Invalid format after '{get_type}', expected variable name and message"),
        }
    }
}
 