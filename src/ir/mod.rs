use crate::{
    phones::Phone,
    rules::conditions::CondType,
    runtime::Command,
    tokens::{Direction, ScopeType, Shift, ShiftType},
    keywords::{
        AND_CHAR, ANY_CHAR, ARG_SEP_CHAR, COMMENT_LINE_START, COND_CHAR,
        DEFINITION_LINE_START, DEFINITION_PREFIX, ESCAPE_CHAR, GAP_STR,
        GET_AS_CODE_LINE_START, GET_LINE_START, INPUT_PATTERN_STR, LABEL_PREFIX,
        LTR_CHAR, MATCH_CHAR, OPTIONAL_END_CHAR, OPTIONAL_START_CHAR, PRINT_LINE_START,
        RTL_CHAR, SELECTION_END_CHAR, SELECTION_START_CHAR, VARIABLE_PREFIX, is_special
    }
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
    Cmd(Command, &'s str),
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
        IrLine::Cmd(Command::Print, args.trim())
    } else if let Some(args) = line.strip_prefix(GET_AS_CODE_LINE_START) {
        // handles get statement
        IrLine::Cmd(Command::GetAsCode, args.trim())
    } else if let Some(args) = line.strip_prefix(GET_LINE_START) {
        // handles get statement
        IrLine::Cmd(Command::Get, args.trim())
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
            _ if escape => if is_special(c) {
                slice.grow(c);
                escape = false;
            } else {
                return Err(IrError::BadEscape(c))
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
            AND_CHAR => push_phone_and(IrToken::Break(Break::And), &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            ANY_CHAR => push_phone_and(IrToken::Any, &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
            ARG_SEP_CHAR => push_phone_and(IrToken::ArgSep, &mut tokens, &mut slice, &mut prefix, tokenization_data)?,
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

    match prefix {
        None if literal == INPUT_PATTERN_STR => tokens.push(IrToken::CondType(CondType::Pattern)),
        None if literal == GAP_STR => tokens.push(IrToken::Gap),
        None if literal.is_empty() => (),
        None => tokens.push(IrToken::Phone(Phone::new(literal))),
        Some(Prefix::Definition) => {
            let content = tokenization_data.get_definition(literal)?;

            for token in content {
                tokens.push(*token);
            }
        },
        Some(Prefix::Label) => tokens.push(IrToken::Label(literal)),
        Some(Prefix::Variable) => {
            let content = tokenization_data.get_variable(literal)?;

            for token in content {
                tokens.push(*token);
            }
        },
    }

    *prefix = None;
    Ok(())
}

/// A wrapper around a str reference that allows slices of it to be taken
/// 
/// The slices may only grow in length or move right to a non intersecting position
/// and with the length being reset
#[derive(Debug, Clone, Copy, PartialEq)]
struct SubString<'s> {
    source: &'s str,
    start: usize,
    len: usize
}

impl<'s> SubString<'s> {
    /// Creates a new `SubString`
    #[inline]
    pub const fn new(source: &'s str) -> Self {
        Self { source, start: 0, len: 0 }
    }

    /// Returns if the substring has 0 length
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Retreives the internal substring (may be done any number of times)
    #[inline]
    pub fn take_slice(&self) -> &'s str {
        &self.source[self.start..self.start + self.len]
    }

    /// Increments the internal substring length by the size of c in utf-8
    #[inline]
    pub const fn grow(&mut self, c: char) {
        self.len += c.len_utf8();
    }

    /// Moves the substring start to the index after the slice ends and resets the length
    #[inline]
    pub const fn move_after (&mut self) {
        self.start += self.len;
        self.len = 0;
    }
    
    /// Moves the substring start to the index after the slice ends and resets the length
    /// then moves skipping a byte
    #[inline]
    pub const fn skip_byte(&mut self) {
        self.move_after();
        self.start += 1;
    }

    /// Moves the substring start to the index after the substring ends and resets the length
    /// then moves skipping a substring the size of c in utf-8
    #[inline]
    pub const fn skip(&mut self, c: char) {
        self.move_after();
        self.start += c.len_utf8();
    }
}

/// Errors that occur when parsing raw text to tokens
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum IrError<'s> {
    EmptyPrefix(Prefix),
    UndefinedDefinition(&'s str),
    UndefinedVariable(&'s str),
    EmptyDefinition,
    BadEscape(char),
}

impl std::error::Error for IrError<'_> {}

impl std::fmt::Display for IrError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPrefix(prefix) => write!(f, "Found prefix '{prefix}' without a following identifier"),
            Self::UndefinedDefinition(name) => write!(f, "Undefined definiton '{DEFINITION_PREFIX}{name}'"),
            Self::UndefinedVariable(name) => write!(f, "Undefined definiton '{VARIABLE_PREFIX}{name}'"),
            Self::EmptyDefinition => write!(f, "Found '{DEFINITION_LINE_START}' with out a following name"),
            Self::BadEscape(c) => write!(f, "Escaped normal character '{c}' ({ESCAPE_CHAR}{c})"),
        }
    }
}
 