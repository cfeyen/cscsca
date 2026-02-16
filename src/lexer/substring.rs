use std::str::Chars;

use crate::lexer::token_types::Span;

/// An accumulating substring
#[derive(Debug, Clone)]
pub(crate) struct Substring<'a> {
    s: &'a str,
    start: usize,
    len: usize,
    iter: Chars<'a>,
    line: usize,
    char: usize,
}

impl<'a> Substring<'a> {
    /// Creates a new `Substring`
    pub fn new(s: &'a str) -> Self {
        Self {
            s,
            start: 0,
            len: 0,
            iter: s.chars(),
            line: 0,
            char: 0,
        }
    }

    /// Gets the substring
    pub fn str(&self) -> &'a str {
        unsafe {
            self.s.get_unchecked(self.start..self.start + self.len)
        }
    }

    /// Gets the index of the start of the substring
    pub const fn start_index(&self) -> usize {
        self.start
    }

    /// Gets the length of the substring
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Gets the line the substring starts on (zero indexed)
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Gets the character index the substring starts on in the line (zero indexed)
    pub const fn char(&self) -> usize {
        self.char
    }

    /// Gets the rest of the source string after the substring
    pub fn rest(&self) -> &'a str {
        unsafe {
            self.s.get_unchecked(self.start + self.len..)
        }
    }

    /// Peeks the next character after the substring
    pub fn peek(&mut self) -> Option<char> {
        self.iter.clone().next()
    }

    /// Peeks `n` characters after the substring
    pub fn peek_past(&self, n: usize) -> Option<char> {
        self.iter.clone().skip(n).next()
    }

    /// Restarts the substring after the character after it
    pub fn skip_char(&mut self) {
        self.grow();
        self.pass();
    }

    /// Grows the substring by a single character
    pub fn grow(&mut self) {
        if let Some(c) = self.iter.next() {
            self.len += c.len_utf8();
        }
    }

    /// Grows the substring by n characters
    pub fn grow_by(&mut self, n: usize) {
        for _ in 0..n {
            self.grow();
        }
    }

    /// Restarts the substring after itself
    pub fn pass(&mut self) -> &'a str {
        let s = self.str();

        for c in s.chars() {
            if c == '\n' {
                self.line += 1;
                self.char = 0;
            } else {
                self.char += 1;
            }
        }

        self.start += self.len;
        self.len = 0;
        s
    }

    /// Determines if the substring is empty and cannot be extended
    pub fn is_exhausted(&self) -> bool {
        self.start == self.s.len()
    }

    /// Gets the span of the substring in the source string
    pub fn span(&self) -> Span {
        Span::new(self.line(), self.char(), self.start_index(), self.len())
    }
}