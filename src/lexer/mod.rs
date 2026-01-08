use crate::{
    ir::{prefix::Prefix, tokens::Break},
    keywords::{
        AND_CHAR, ANY_CHAR, ARG_SEP_CHAR, BOUND_CHAR, COMMENT_LINE_START, COND_CHAR, DEFINITION_LINE_START,
        DEFINITION_PREFIX, ESCAPE_CHAR, GET_AS_CODE_LINE_START, GET_LINE_START, LABEL_PREFIX,
        LAZY_DEFINITION_LINE_START, LTR_CHAR, MATCH_CHAR, NOT_CHAR, OPTIONAL_END_CHAR, OPTIONAL_START_CHAR,
        PRINT_LINE_START, REPETITION_END_CHAR, REPETITION_START_CHAR, RTL_CHAR, SELECTION_END_CHAR,
        SELECTION_START_CHAR, VARIABLE_PREFIX, is_isolated_char, is_isolation_bound, is_special_char, is_special_str
    },
    lexer::{sir::SirToken, substring::Substring, token_types::{PhoneValidStr, Span}},
    tokens::{AndType, CondType, Direction, ScopeType, Shift, ShiftType}
};

pub(crate) mod substring;
pub mod token_types;
pub mod sir;
#[cfg(test)]
mod tests;

/// An `Iterator` over the Scoped Intermediate Representation of the source code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sir<'s> {
    line: usize,
    tokens: Vec<SirToken<'s>>,
}

impl<'s> Sir<'s> {
    /// Creates a new SIR iterator from a list of `SirTokens`
    pub fn new(mut sir: Vec<SirToken<'s>>) -> Self {
        sir.reverse();

        Self {
            line: 0,
            tokens: sir,
        }
    }

    /// Determines if the iterator is exausted
    pub const fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Gets the current line number (starts at zero)
    pub const fn line(&self) -> usize {
        self.line
    }

    #[cfg(feature = "debug_tokens")]
    /// Peeks the next token
    pub fn peek(&self) -> Option<&<Self as Iterator>::Item> {
        self.tokens.last()
    }
}

impl<'s> Iterator for Sir<'s> {
    type Item = SirToken<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.tokens.pop();

        if let Some(SirToken::EndOfExpr(_) | SirToken::NonPhoneEscape('\n', _)) = token {
            self.line += 1;
        }

        token
    }
}

/// A struct from parsing rules into SIR
#[derive(Debug)]
pub struct Lexer<'s> {
    accumulator: Substring<'s>,
    prefix: Option<Prefix>,
    tokens: Vec<SirToken<'s>>,
}

/// A helper macro for pushing single-character tokens
macro_rules! push_char_token {
    ($self:ident <- $var:ident $(($($expr:expr),* $(,)?))?) => {
        {
            $self.push_phone();
            $self.accumulator.grow();
            $self.tokens.push(SirToken::$var($($($expr,)*)? $self.accumulator.span()));
            $self.accumulator.pass();
        }
    };
}

impl<'s> Lexer<'s> {
    /// Creates a new `Lexer` from rules
    fn new(rules: &'s str) -> Self {
        Self { accumulator: Substring::new(rules), prefix: None, tokens: Vec::new() }
    }

    /// Converts rules into SIR
    pub fn lex(rules: &'s str) -> Sir<'s> {
        let mut lexer = Self::new(rules);

        while !lexer.accumulator.is_exhausted() {
            lexer.partial_parse_token();
        }

        Sir::new(lexer.tokens)
    }

    /// Attempts to grow the accumulator by at least one character and handles creating any tokens that must be created
    fn partial_parse_token(&mut self) {
        if let Some(c) = self.accumulator.peek() {
            let empty_acc = self.accumulator.str().is_empty();
            let at_line_start = self.accumulator.char() == 0 && self.accumulator.len() == 0;
            let rest = self.accumulator.rest();

            match c {
                // handles statements
                _ if at_line_start && rest.starts_with(LAZY_DEFINITION_LINE_START) => {
                    self.accumulator.grow_by(LAZY_DEFINITION_LINE_START.len());
                    self.tokens.push(SirToken::LazyDefinitionDeclaration(self.accumulator.span()));
                    _ = self.accumulator.pass();
                },
                _ if at_line_start && rest.starts_with(DEFINITION_LINE_START) => {
                    self.accumulator.grow_by(DEFINITION_LINE_START.len());
                    self.tokens.push(SirToken::DefinitionDeclaration(self.accumulator.span()));
                    _ = self.accumulator.pass();
                },
                _ if at_line_start && rest.starts_with(PRINT_LINE_START) => {
                    self.accumulator.grow_by(PRINT_LINE_START.len());
                    self.tokens.push(SirToken::PrintCommand(self.accumulator.span()));
                    _ = self.accumulator.pass();

                    let (msg, span) = self.rest_of_line_as_str();
                    self.tokens.push(SirToken::Message(msg.trim_start(), span));
                },
                _ if at_line_start && rest.starts_with(GET_AS_CODE_LINE_START) => {
                    self.accumulator.grow_by(GET_AS_CODE_LINE_START.len());
                    self.tokens.push(SirToken::GetAsCodeCommand(self.accumulator.span()));
                    _ = self.accumulator.pass();

                    if self.parse_phone() {
                        let (msg, span) = self.rest_of_line_as_str();
                        self.tokens.push(SirToken::Message(msg.trim_start(), span));
                    }
                },
                _ if at_line_start && rest.starts_with(GET_LINE_START) => {
                    self.accumulator.grow_by(GET_LINE_START.len());
                    self.tokens.push(SirToken::GetCommand(self.accumulator.span()));
                    _ = self.accumulator.pass();

                    if self.parse_phone() {
                        let (msg, span) = self.rest_of_line_as_str();
                        self.tokens.push(SirToken::Message(msg.trim_start(), span));
                    }
                },
                _ if at_line_start && rest.starts_with(COMMENT_LINE_START) => {
                    let (_, span) = self.rest_of_line_as_str();
                    self.tokens.push(SirToken::Comment(span));
                },
                // handles escapes
                ESCAPE_CHAR => {
                    if let Some(c2) = self.accumulator.peek_past(1) {
                        if c2 == '\r' && let Some('\n') = self.accumulator.peek_past(2) {
                            self.push_phone();
                            self.accumulator.grow_by(3);
                            self.tokens.push(SirToken::NonPhoneEscape('\n', self.accumulator.span()));
                            _ = self.accumulator.pass();
                        } else if self.is_escapable(c2, 1, empty_acc) {
                            self.accumulator.grow_by(2);
                        } else {
                            self.push_phone();
                            self.accumulator.grow_by(2);
                            self.tokens.push(SirToken::NonPhoneEscape(c2, self.accumulator.span()));
                            _ = self.accumulator.pass();
                        }
                    } else {
                        self.push_phone();
                        self.accumulator.grow();
                        self.tokens.push(SirToken::NonPhoneEscape('\n', self.accumulator.span()));
                        _ = self.accumulator.pass();
                    }
                },
                // handles prefixes
                DEFINITION_PREFIX => self.set_prefix(Prefix::Definition),
                LABEL_PREFIX => self.set_prefix(Prefix::Label),
                VARIABLE_PREFIX => self.set_prefix(Prefix::Variable),
                // handles scope bounds
                OPTIONAL_START_CHAR => push_char_token!(self <- ScopeStart(ScopeType::Optional)),
                OPTIONAL_END_CHAR => push_char_token!(self <- ScopeEnd(ScopeType::Optional)),
                SELECTION_START_CHAR => push_char_token!(self <- ScopeStart(ScopeType::Selection)),
                SELECTION_END_CHAR => push_char_token!(self <- ScopeEnd(ScopeType::Selection)),
                REPETITION_START_CHAR => push_char_token!(self <- ScopeStart(ScopeType::Repetition)),
                REPETITION_END_CHAR => push_char_token!(self <- ScopeEnd(ScopeType::Repetition)),
                // handles other single-character tokens
                AND_CHAR => push_char_token!(self <- Break(Break::And(AndType::And))),
                ANY_CHAR => push_char_token!(self <- Any),
                ARG_SEP_CHAR => push_char_token!(self <- ArgSep),
                BOUND_CHAR => push_char_token!(self <- Bound),
                MATCH_CHAR => push_char_token!(self <- CondFocus(CondType::Match)),
                // handles compoundable characters
                NOT_CHAR if empty_acc => match self.tokens.last_mut() {
                    Some(SirToken::Break(Break::And(and_type @ AndType::And), span)) => {
                        *and_type = AndType::AndNot;
                        span.lengthen(c);
                        self.accumulator.skip_char();
                    },
                    Some(SirToken::Break(b @ Break::Cond, span)) => {
                        *b = Break::AntiCond;
                        span.lengthen(c);
                        self.accumulator.skip_char();
                    },
                    _ => push_char_token!(self <- Negative),
                },
                NOT_CHAR => push_char_token!(self <- Negative),
                LTR_CHAR => if empty_acc && let Some(SirToken::Break(Break::Shift(Shift {
                    dir: Direction::Ltr,
                    kind: kind @ ShiftType::Stay
                }), span)) = self.tokens.last_mut() {
                    *kind = ShiftType::Move;
                    span.lengthen(c);
                    self.accumulator.skip_char();
                } else {
                    push_char_token!(self <- Break(Break::Shift(Shift {
                        dir: Direction::Ltr,
                        kind: ShiftType::Stay
                    })));
                },
                RTL_CHAR => if empty_acc && let Some(SirToken::Break(Break::Shift(Shift {
                    dir: Direction::Rtl,
                    kind: kind @ ShiftType::Stay
                }), span)) = self.tokens.last_mut() {
                    *kind = ShiftType::Move;
                    span.lengthen(c);
                    self.accumulator.skip_char();
                } else {
                    push_char_token!(self <- Break(Break::Shift(Shift {
                        dir: Direction::Rtl,
                        kind: ShiftType::Stay
                    })));
                },
                COND_CHAR => if empty_acc && let Some(SirToken::Break(b @ Break::Cond, span)) = self.tokens.last_mut() {
                    *b = Break::AntiCond;
                    span.lengthen(c);
                    self.accumulator.skip_char();
                } else {
                    push_char_token!(self <- Break(Break::Cond));
                },
                // handles whitespace
                '\n' => push_char_token!(self <- EndOfExpr),
                _ if c.is_whitespace() => if empty_acc && let Some(SirToken::Whitespace(span)) = self.tokens.last_mut() {
                    span.lengthen(c);
                    self.accumulator.skip_char();
                } else {
                    push_char_token!(self <- Whitespace);
                }
                // handles other characters
                _ => self.accumulator.grow(),
            }
        } else if !self.accumulator.str().is_empty() {
            self.push_phone();
        }
    }

    /// Determines if the character `c` located `c_offset` after the accumulator is escapeable
    fn is_escapable(&self, c: char, c_offset: usize, proceeded_by_bound: bool) -> bool {
        is_special_char(c) || (is_isolated_char(c) && (proceeded_by_bound && self.is_right_bound(c_offset + 1)))
    }

    /// Determines if the character located `offset` after the accumulator is a boundary on the right side of special sting
    fn is_right_bound(&self, offset: usize) -> bool {
        if let Some(c) = self.accumulator.peek_past(offset) {
            is_isolation_bound(c) || (c == ESCAPE_CHAR && if let Some(c2) = self.accumulator.peek_past(offset + 1) {
                c2 == '\n'
            } else {
                true
            })
        } else {
            true
        }
    }

    /// Skips past the accumulator and the following whitespace and parses the next phone
    /// 
    /// returns `true` if a phone is parsed
    fn parse_phone(&mut self) -> bool {
        // skips whitespace
        while let Some(c) = self.accumulator.peek() && c != '\n' && c.is_whitespace() {
            self.accumulator.grow();
        }

        // pushes a whitespace token and clears the accumulator
        if !self.accumulator.str().is_empty() {
            self.tokens.push(SirToken::Whitespace(self.accumulator.span()));
            _ = self.accumulator.pass();
        }

        // copys the accumulator then finds the next phone if any
        let mut forward_acc = self.accumulator.clone();

        while let Some(c) = forward_acc.peek() {
            match c {
                ESCAPE_CHAR => if let Some(c2) = forward_acc.peek_past(1) {
                    if self.is_escapable(c2, 2, forward_acc.len() == 0) {
                        forward_acc.grow_by(2);
                    } else {
                        break;
                    }
                }
                _ if c.is_whitespace() || is_special_char(c) => break,
                _ => (),
            }

            forward_acc.grow();
        }

        let s = forward_acc.str();

        // adds any found phone and catches the accumulator up to the forward accumulator if needed
        if s.is_empty() || is_special_str(s) {
            return false;
        } else {
            self.accumulator = forward_acc;
            self.tokens.push(SirToken::Phone(PhoneValidStr::new(
                s,
                self.accumulator.line(),
                self.accumulator.char(),
                self.accumulator.start_index()
            )));
            _ = self.accumulator.pass();
            true
        }
    } 

    // Sets the accumulator's prefix
    fn set_prefix(&mut self, prefix: Prefix) {
        self.push_phone();
        self.prefix = Some(prefix);
        self.accumulator.skip_char();
    }

    // Pushes the current accumulator as a phone, prefixed name, or special string
    fn push_phone(&mut self) {
        // gets the accumulator position data
        let line = self.accumulator.line();
        let mut char = self.accumulator.char();
        let mut index = self.accumulator.start_index();
        let mut len = self.accumulator.len();

        // accounts for any prefix
        if self.prefix.is_some() {
            char -= 1;
            index -= 1;
            len += 1;
        }

        let s = self.accumulator.pass();

        // pushes the accumulator
        if s.is_empty() {
            if let Some(prefix) = self.prefix {
                self.tokens.push(SirToken::InvalidPrefix(prefix, Span::new(line, char, index, len)));
            }
        } else {
            let fvs = PhoneValidStr::new_with_len(s, line, char, index, len);

            let token = match self.prefix {
                Some(Prefix::Definition) => SirToken::Definition(fvs),
                Some(Prefix::Label) => SirToken::Label(fvs),
                Some(Prefix::Variable) => SirToken::Variable(fvs),
                None if is_special_str(s) => SirToken::SpecialStr(fvs),
                None => SirToken::Phone(fvs),
            };

            self.tokens.push(token);
        }

        self.prefix = None;
    }

    // Parses the rest of the line as a string
    fn rest_of_line_as_str(&mut self) -> (&'s str, Span) {
        while let Some(c) = self.accumulator.peek() && c != '\n' {
            self.accumulator.grow();
        }

        let span = self.accumulator.span();
        let str = self.accumulator.pass();

        (str, span)
    }
}