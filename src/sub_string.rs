/// A wrapper around a str reference that allows slices of it to be taken
/// 
/// The slices may only grow in length or move right to a non intersecting position
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