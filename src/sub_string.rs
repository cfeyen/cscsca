/// A wrapper around a str reference that allows slices of it to be taken
/// 
/// The slices may only grow in length, be reduced by a character it ends with, or move right to a non intersecting position
/// and with the length being reset
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubString<'s> {
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

    /// Moves the substring start to the index after the substring ends and resets the length
    /// then moves skipping a substring the size of c in utf-8
    #[inline]
    pub const fn skip(&mut self, c: char) {
        self.move_after();
        self.start += c.len_utf8();
    }

    /// Shrinks the substring by the length of a character if it ends with it
    /// 
    /// Returns `true` if the substring was shrunk
    pub fn shrink(&mut self, c: char) -> bool {
        if self.take_slice().ends_with(c) {
            self.len -= c.len_utf8();
            true
        } else {
            false
        }
    }
}