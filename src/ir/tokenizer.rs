use std::num::NonZero;

use crate::{
    escaped_strings::check_escapes,
    executor::io_events::{GetType, IoEvent, RuntimeIoEvent, TokenizerIoEvent},
    ir::{prefix::Prefix, tokenization_data::TokenizationData, tokens::{Break, IrToken}, IrError, IrLine},
    keywords::{is_special_char, AND_CHAR, ANY_CHAR, ARG_SEP_CHAR, BOUND_CHAR, COMMENT_LINE_START, COND_CHAR, DEFINITION_LINE_START, DEFINITION_PREFIX, ESCAPE_CHAR, GAP_STR, GET_AS_CODE_LINE_START, GET_LINE_START, INPUT_PATTERN_STR, LABEL_PREFIX, LTR_CHAR, MATCH_CHAR, NOT_CHAR, OPTIONAL_END_CHAR, OPTIONAL_START_CHAR, PRINT_LINE_START, RTL_CHAR, SELECTION_END_CHAR, SELECTION_START_CHAR, SPECIAL_STRS, VARIABLE_PREFIX}, phones::Phone, rules::conditions::{AndType, CondType},
    sub_string::SubString,
    tokens::{Direction, ScopeType, Shift, ShiftType},
};

/// Converts source code into intermediate representation tokens
pub fn tokenize_line_or_create_command<'s>(line: &'s str, rem_lines: &mut impl Iterator<Item = &'s str>, tokenization_data: &mut TokenizationData<'s>) -> Result<IrLine<'s>, IrError<'s>> {
    let ir_line = if line.starts_with(COMMENT_LINE_START) {
        // handles comments
        IrLine::Empty
    } else if let Some(definition_content) = line.strip_prefix(DEFINITION_LINE_START) {
        // handles definitions
        let (mut ir, mut continues_on_next_line) = tokenize_line(definition_content, tokenization_data)?;

        while continues_on_next_line && let Some(next_line) = rem_lines.next() {
            let (mut next_ir, continue_again) = tokenize_line(next_line, tokenization_data)?;
            ir.append(&mut next_ir);
            continues_on_next_line = continue_again;
        }

        if let Some(IrToken::Phone(name)) = ir.first() {
            tokenization_data.set_definition(name.as_str(), ir[1..].into());
            IrLine::Empty
        } else {
            return Err(IrError::EmptyDefinition);
        }
    } else if let Some(args) = line.strip_prefix(PRINT_LINE_START) {
        // handles print statement
        IrLine::IoEvent(IoEvent::Runtime(RuntimeIoEvent::Print { msg: args.trim() }))
    } else if let Some(args) = line.strip_prefix(GET_AS_CODE_LINE_START) {
        // handles get as code
        if let Some((var, msg)) = args.trim().split_once(char::is_whitespace) {
            IrLine::IoEvent(IoEvent::Tokenizer(TokenizerIoEvent::Get {
                get_type: GetType::Code,
                var,
                msg: msg.trim(),
            }))
        } else {
            return Err(IrError::InvalidGetFormat(GetType::Code));
        }
    } else if let Some(args) = line.strip_prefix(GET_LINE_START) {
        // handles get
        if let Some((var, msg)) = args.trim().split_once(char::is_whitespace) {
            IrLine::IoEvent(IoEvent::Tokenizer(TokenizerIoEvent::Get {
                get_type: GetType::Phones,
                var,
                msg: msg.trim(),
            }))
        } else {
            return Err(IrError::InvalidGetFormat(GetType::Phones));
        }
    } else {
        // handles rules
        let (mut ir, mut continues_on_next_line) = tokenize_line(line, tokenization_data)?;

        let mut line_count = 1;

        while continues_on_next_line && let Some(next_line) = rem_lines.next() {
                let (mut next_ir, continue_again) = tokenize_line(next_line, tokenization_data)?;
                ir.append(&mut next_ir);
                continues_on_next_line = continue_again;
                line_count += 1;
        }

        let mut ir_line = IrLine::Ir {
            tokens: ir,
            // Safety: `line_count` starts at one and is only incremented
            lines: unsafe { NonZero::new_unchecked(line_count) }
        };
        
        // converts empty rules to the empty varient
        if let IrLine::Ir { tokens, lines } = &ir_line && tokens.is_empty() && lines.get() == 1 {
            ir_line = IrLine::Empty;
        }

        ir_line
    };

    Ok(ir_line)
}

/// Converts a line to tokens
pub fn tokenize_line<'s>(line: &'s str, tokenization_data: &TokenizationData<'s>) -> Result<(Vec<IrToken<'s>>, bool), IrError<'s>> {
    let chars = line.chars().peekable();
    let mut tokens = Vec::new();
    let mut prefix = None;
    let mut slice = SubString::new(line);
    let mut escape = false;

    for c in chars {
        parse_character(c, &mut tokens, &mut prefix, &mut slice, &mut escape, tokenization_data)?;
    }

    if escape {
        _ = slice.shrink(ESCAPE_CHAR);
    }

    push_phone(&mut tokens, &mut slice, &mut prefix, tokenization_data)?;

    Ok((tokens, escape))
}

/// Handles a signle character
fn parse_character<'s>(c: char, tokens: &mut Vec<IrToken<'s>>, prefix: &mut Option<Prefix>, slice: &mut SubString<'s>, escape: &mut bool, tokenization_data: &TokenizationData<'s>) -> Result<(), IrError<'s>> {
    match c {
        // handles escapes
        _ if *escape => {
            slice.grow(c);
            *escape = false;
        },
        ESCAPE_CHAR => {
            slice.grow(c);
            *escape = true;
        },
        // handles prefixes
        DEFINITION_PREFIX => start_prefix(Prefix::Definition, tokens, slice, prefix, tokenization_data)?,
        LABEL_PREFIX => start_prefix(Prefix::Label, tokens, slice, prefix, tokenization_data)?,
        VARIABLE_PREFIX => start_prefix(Prefix::Variable, tokens, slice, prefix, tokenization_data)?,
        // handles scope bounds
        OPTIONAL_START_CHAR => push_phone_and(c, IrToken::ScopeStart(ScopeType::Optional), tokens, slice, prefix, tokenization_data)?,
        OPTIONAL_END_CHAR => push_phone_and(c, IrToken::ScopeEnd(ScopeType::Optional), tokens, slice, prefix, tokenization_data)?,
        SELECTION_START_CHAR => push_phone_and(c, IrToken::ScopeStart(ScopeType::Selection), tokens, slice, prefix, tokenization_data)?,
        SELECTION_END_CHAR => push_phone_and(c, IrToken::ScopeEnd(ScopeType::Selection), tokens, slice, prefix, tokenization_data)?,
        // handles simple one-to-one char to token pushes
        AND_CHAR => push_phone_and(c, IrToken::Break(Break::And(AndType::And)), tokens, slice, prefix, tokenization_data)?,
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

            push_phone_and(c, token, tokens, slice, prefix, tokenization_data)?;
        }
        ANY_CHAR => push_phone_and(c, IrToken::Any, tokens, slice, prefix, tokenization_data)?,
        ARG_SEP_CHAR => push_phone_and(c, IrToken::ArgSep, tokens, slice, prefix, tokenization_data)?,
        BOUND_CHAR => push_phone_and(c, IrToken::Phone(Phone::Bound), tokens, slice, prefix, tokenization_data)?,
        MATCH_CHAR => push_phone_and(c, IrToken::CondType(CondType::Match), tokens, slice, prefix, tokenization_data)?,
        // handles compound char to token pushes
        LTR_CHAR => {
            let kind = if let Some(IrToken::Break(Break::Shift(Shift { dir: Direction::Ltr, kind: ShiftType::Stay }))) = tokens.last() {
                tokens.pop();
                ShiftType::Move
            } else {
                ShiftType::Stay
            };

            push_phone_and(c, IrToken::Break(Break::Shift(Shift { dir: Direction::Ltr, kind })), tokens, slice, prefix, tokenization_data)?;
        },
        RTL_CHAR => {
            let kind = if let Some(IrToken::Break(Break::Shift(Shift { dir: Direction::Rtl, kind: ShiftType::Stay }))) = tokens.last() {
                tokens.pop();
                ShiftType::Move
            } else {
                ShiftType::Stay
            };

            push_phone_and(c, IrToken::Break(Break::Shift(Shift { dir: Direction::Rtl, kind })), tokens, slice, prefix, tokenization_data)?;
        },
        COND_CHAR => {
            let cond_type = if let Some(IrToken::Break(Break::Cond)) = tokens.last() {
                tokens.pop();
                Break::AntiCond
            } else {
                Break::Cond
            };

            push_phone_and(c, IrToken::Break(cond_type), tokens, slice, prefix, tokenization_data)?;
        },
        // whitespace
        _ if c.is_whitespace() => {
            push_phone(tokens, slice, prefix, tokenization_data)?;
            slice.skip(c);
        },
        // other chars
        _ => slice.grow(c)
    }

    Ok(())
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
            slice.skip(new_prefix.char());

            *prefix = Some(new_prefix);
            Ok(())
        }
    }
}

/// Pushes the slice according to `push_phone`, then pushes the provided token
/// and moves the slice over an extra character to account for that token
fn push_phone_and<'s>(c: char, token: IrToken<'s>, tokens: &mut Vec<IrToken<'s>>, slice: &mut SubString<'s>, prefix: &mut Option<Prefix>, tokenization_data: &TokenizationData<'s>) -> Result<(), IrError<'s>> {
    push_phone(tokens, slice, prefix, tokenization_data)?;
    slice.skip(c);
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

    let mut escaped = false;

    for c in input.chars() {
        match c {
            ESCAPE_CHAR if !escaped => escaped = true,
            _ if is_special_char(c) && !escaped
                => return Err(IrError::ReservedCharacter(c)),
            _ => escaped = false,
        }
    }

    Ok(())
}