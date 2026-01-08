/// A struct that contains the information for where in a string a token occurs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    index: usize,
    line: usize,
    char: usize,
    len: usize,
}

impl Span {
    /// Creates a new `Span`
    pub(super) const fn new(line: usize, char: usize, index: usize, len: usize) -> Self {
        Self { index, line, char, len }
    }

    /// Extends the span's length by one
    pub(super) const fn lengthen(&mut self, c: char) {
        self.len += c.len_utf8();
    }
}

#[cfg(feature = "debug_tokens")]
impl Span {
    /// Gets the line the span starts on (zero indexed)
    pub const fn line(&self) -> usize { self.line }

    /// Gets the character of the line the span starts on (zero indexed)
    pub const fn char(&self) -> usize { self.char }

    /// Gets the index of the starting byte of the span
    pub const fn index(&self) -> usize { self.index }

    /// Gets the length of the span in bytes
    pub const fn len(&self) -> usize { self.len }
}

/// A scoped string that may be a phone or prefixed name
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhoneValidStr<'s> {
    str: &'s str,
    span: Span
}

impl<'s> PhoneValidStr<'s> {
    pub(super) const fn new(s: &'s str, line: usize, char: usize, index: usize) -> Self {
        Self::new_with_len(s, line, char, index, s.len())
    }

    pub(super) const fn new_with_len(s: &'s str, line: usize, char: usize, index: usize, len: usize) -> Self {
        Self {
            str: s,
            span: Span { index, line, char, len }
        }
    }
    
    /// Gets the string
    pub const fn str(&self) -> &'s str { self.str }

    #[cfg(feature = "debug_tokens")]
    /// Gets the span of the phone or name
    pub const fn span(&self) -> &Span { &self.span }
}

