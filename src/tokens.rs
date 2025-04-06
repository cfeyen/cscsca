use crate::{
    meta_tokens::{Direction, ScopeType, Shift, ShiftType, LTR_CHAR, OPTIONAL_END_CHAR, OPTIONAL_START_CHAR, RTL_CHAR, SELECTION_END_CHAR, SELECTION_START_CHAR},
    rules::conditions::{CondType, EQUALITY_CHAR, INPUT_STR},
    commands::Command,
};
use compile_time_data::CompileTimeData;
use ir::{Break, IrToken, ANY_CHAR, ARG_SEP_CHAR, COND_CHAR, GAP_STR, AND_CHAR};
use prefix::{Prefix, DEFINITION_PREFIX, SELECTION_PREFIX, VARIABLE_PREFIX};

pub mod ir;
pub mod prefix;
pub mod token_checker;
pub mod compile_time_data;

#[cfg(test)]
mod tests;

pub const DEFINITION_LINE_START: &str = "DEFINE";
pub const PRINT_LINE_START: &str = "PRINT";
pub const GET_LINE_START: &str = "GET";
pub const COMMENT_LINE_START: &str = "##";
pub const ESCAPE_CHAR: char = '\\';

/// A list of IrTokens, a command, or nothing representing a line of source code
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum IrLine<'s> {
    Ir(Vec<IrToken<'s>>),
    Cmd(Command, &'s str),
    Empty,
}

/// Converts source code into intermediate representation tokens
/// 
/// Note: these tokens may not be structurally valid and should be checked
pub fn tokenize_line_or_create_command<'s>(line: &'s str, compile_time_data: &mut CompileTimeData<'s>) -> Result<IrLine<'s>, IrError<'s>> {
    Ok(if let Some(definition_content) = line.strip_prefix(DEFINITION_LINE_START) {
        // handles definitions
        let ir = tokenize_line(definition_content, compile_time_data)?;

        if let Some(IrToken::Phone(name)) = ir.first() {
            compile_time_data.definitions.insert(name, ir[1..].into());
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
    } else if let Some(args) = line.strip_prefix(GET_LINE_START) {
        // handles get statement
        IrLine::Cmd(Command::Get, args.trim())
    } else {
        // handles rules
        let mut ir_line = IrLine::Ir(tokenize_line(line, compile_time_data)?);
        
        // converts empty rules to the empty varient
        if let IrLine::Ir(ir) = &ir_line {
            if ir.is_empty() { ir_line = IrLine::Empty; }
        }

        ir_line
    })
}

/// Converts a line to tokens
fn tokenize_line<'s>(line: &'s str, compile_time_data: &CompileTimeData<'s>) -> Result<Vec<IrToken<'s>>, IrError<'s>> {
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
                escape = true
            },
            // handles prefixes
            DEFINITION_PREFIX => start_prefix(Prefix::Definition, &mut tokens, &mut slice, &mut prefix, compile_time_data)?,
            SELECTION_PREFIX => start_prefix(Prefix::Label, &mut tokens, &mut slice, &mut prefix, compile_time_data)?,
            VARIABLE_PREFIX => start_prefix(Prefix::Variable, &mut tokens, &mut slice, &mut prefix, compile_time_data)?,
            // handles scope bounds
            OPTIONAL_START_CHAR => push_phone_and(IrToken::ScopeStart(ScopeType::Optional), &mut tokens, &mut slice, &mut prefix, compile_time_data)?,
            OPTIONAL_END_CHAR => push_phone_and(IrToken::ScopeEnd(ScopeType::Optional), &mut tokens, &mut slice, &mut prefix, compile_time_data)?,
            SELECTION_START_CHAR => push_phone_and(IrToken::ScopeStart(ScopeType::Selection), &mut tokens, &mut slice, &mut prefix, compile_time_data)?,
            SELECTION_END_CHAR => push_phone_and(IrToken::ScopeEnd(ScopeType::Selection), &mut tokens, &mut slice, &mut prefix, compile_time_data)?,
            // handles simple one-to-one char to token pushes
            AND_CHAR => push_phone_and(IrToken::Break(Break::And), &mut tokens, &mut slice, &mut prefix, compile_time_data)?,
            ANY_CHAR => push_phone_and(IrToken::Any, &mut tokens, &mut slice, &mut prefix, compile_time_data)?,
            ARG_SEP_CHAR => push_phone_and(IrToken::ArgSep, &mut tokens, &mut slice, &mut prefix, compile_time_data)?,
            EQUALITY_CHAR => push_phone_and(IrToken::CondType(CondType::Equality), &mut tokens, &mut slice, &mut prefix, compile_time_data)?,
            // handles compound char to token pushes
            LTR_CHAR => {
                let kind = if let Some(IrToken::Break(Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay }))) = tokens.last() {
                    tokens.pop();
                    ShiftType::Move
                } else {
                    ShiftType::Stay
                };

                push_phone_and(IrToken::Break(Break::Shift(Shift { dir: Direction::LTR, kind })), &mut tokens, &mut slice, &mut prefix, compile_time_data)?;
            },
            RTL_CHAR => {
                let kind = if let Some(IrToken::Break(Break::Shift(Shift { dir: Direction::RTL, kind: ShiftType::Stay }))) = tokens.last() {
                    tokens.pop();
                    ShiftType::Move
                } else {
                    ShiftType::Stay
                };

                push_phone_and(IrToken::Break(Break::Shift(Shift { dir: Direction::RTL, kind })), &mut tokens, &mut slice, &mut prefix, compile_time_data)?;
            },
            COND_CHAR => {
                let cond_type = if let Some(IrToken::Break(Break::Cond)) = tokens.last() {
                    tokens.pop();
                    Break::AntiCond
                } else {
                    Break::Cond
                };

                push_phone_and(IrToken::Break(cond_type), &mut tokens, &mut slice, &mut prefix, compile_time_data)?;
            },
            // whitespace
            _ if c.is_whitespace() => {
                push_phone(&mut tokens, &mut slice, &mut prefix, compile_time_data)?;
                slice.skip(c);
            },
            // other chars
            _ => slice.grow(c)
        }
    }

    push_phone(&mut tokens, &mut slice, &mut prefix, compile_time_data)?;

    Ok(tokens)
}


/// Pushes the slice according to `push_phone`, then sets the prefix
/// and moves the slice over an extra character to account for the prefix character
fn start_prefix<'s>(new_prefix: Prefix, tokens: &mut Vec<IrToken<'s>>, slice: &mut SubString<'s>, prefix: &mut Option<Prefix>, compile_time_data: &CompileTimeData<'s>) -> Result<(), IrError<'s>> {
    if prefix.is_some() && slice.is_empty() { 
        return Err(IrError::EmptyPrefix(prefix.unwrap()))
    } else {
        push_phone(tokens, slice, prefix, compile_time_data)?;
        slice.skip_byte();
    }

    *prefix = Some(new_prefix);
    Ok(())
}

/// Pushes the slice according to `push_phone`, then pushes the provided token
/// and moves the slice over an extra character to account for that token
fn push_phone_and<'s>(token: IrToken<'s>, tokens: &mut Vec<IrToken<'s>>, slice: &mut SubString<'s>, prefix: &mut Option<Prefix>, compile_time_data: &CompileTimeData<'s>) -> Result<(), IrError<'s>> {
    push_phone(tokens, slice, prefix, compile_time_data)?;
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
fn push_phone<'s>(tokens: &mut Vec<IrToken<'s>>, slice: &mut SubString<'s>, prefix: &mut Option<Prefix>, compile_time_data: &CompileTimeData<'s>) -> Result<(), IrError<'s>> {
    let literal = slice.take_slice();
    slice.move_after();

    match prefix {
        None if literal == INPUT_STR => tokens.push(IrToken::CondType(CondType::MatchInput)),
        None if literal == GAP_STR => tokens.push(IrToken::Gap),
        None if literal.is_empty() => (),
        None => tokens.push(IrToken::Phone(literal)),
        Some(Prefix::Definition) => {
            if let Some(content) = compile_time_data.definitions.get(literal) {
                for token in content {
                    tokens.push(*token);
                }
            } else {
                return Err(IrError::UndefinedDefinition(literal))
            }
        },
        Some(Prefix::Label) => tokens.push(IrToken::Label(literal)),
        Some(Prefix::Variable) => {
            let content = compile_time_data.get_variable(literal)?;

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
    /// Creates a new SliceData
    #[inline]
    pub const fn new(source: &'s str) -> Self {
        Self { source, start: 0, len: 0 }
    }

    /// Returns if the slice has 0 length
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Retreives the internal slice (may be done any number of times)
    #[inline]
    pub fn take_slice(&self) -> &'s str {
        &self.source[self.start..self.start + self.len]
    }

    /// Increments the internal slice length by the size of c in utf-8
    #[inline]
    pub const fn grow(&mut self, c: char) {
        self.len += c.len_utf8();
    }

    /// Moves the slice start to the index after the slice ends and resets the length
    #[inline]
    pub const fn move_after (&mut self) {
        self.start += self.len;
        self.len = 0;
    }
    
    /// Moves the slice start to the index after the slice ends and resets the length
    /// then moves skipping a byte
    #[inline]
    pub const fn skip_byte(&mut self) {
        self.move_after();
        self.start += 1;
    }

    /// Moves the slice start to the index after the slice ends and resets the length
    /// then moves skipping a slice the size of c in utf-8
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
}

impl std::error::Error for IrError<'_> {}

impl std::fmt::Display for IrError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::EmptyPrefix(prefix) => format!("Found prefix '{prefix}' without a following identifier"),
            Self::UndefinedDefinition(name) => format!("Undefined definiton '{DEFINITION_PREFIX}{name}'"),
            Self::UndefinedVariable(name) => format!("Undefined definiton '{VARIABLE_PREFIX}{name}'"),
            Self::EmptyDefinition => format!("Found '{DEFINITION_LINE_START}' with out a following name"),
        };

        write!(f, "{}", s)
    }
}
 