use crate::{
    escaped_strings::EscapedStr,
    keywords::{char_to_str, BOUND_CHAR, ESCAPE_CHAR},
    matcher::{
        choices::{Choices, OwnedChoices},
        match_state::UnitState,
        phones::Phones
    },
    lexer::substring::Substring,
};

#[cfg(test)]
mod tests;

/// A representation of a phoneme or word boundary
/// 
/// Stores the phoneme's symbol as a reference to the origional text or rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phone<'s> {
    /// A symbol representing a phoneme
    Symbol(&'s str),
    /// A word boundary
    #[default]
    Bound,
}

impl<'s> Phone<'s> {
    /// Returns the phone's symbol.
    /// If the phone is a boundary, `" "` (space) is returned
    #[must_use]
    pub const fn as_str(&self) -> &'s str {
        match self {
            Self::Symbol(symbol) => symbol,
            Self::Bound => " ",
        }
    }

    
    /// Returns the phone's symbol.
    /// If the phone is a boundary, `BOUND_CHAR` is returned as a string
    #[must_use]
    pub const fn as_symbol(&self) -> &'s str {
        match self {
            Phone::Symbol(symbol) => symbol,
            Phone::Bound => const { char_to_str(&BOUND_CHAR) },
        }
    }

    /// Determines if a phone matches a bound
    #[must_use]
    pub fn is_bound(&self) -> bool {
        self.matches(&Self::Bound)
    }

    /// Determines if two phones match
    /// 
    /// Equal phones match and bounds and all-whitespace phones match
    #[must_use]
    pub fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bound, Self::Symbol(symbol)) | (Self::Symbol(symbol), Self::Bound)
                => symbol.chars().all(char::is_whitespace),
            _ => self == other,
        }
    }
}

impl<'s> UnitState<'s> for Phone<'s> {
    fn matches<'p>(&self, phones: &mut Phones<'_, 'p>, _: &Choices<'_, 'p>) -> Option<OwnedChoices<'s>> where 's: 'p {
        let matches = Phone::matches(self, phones.next());

        if matches {
            Some(OwnedChoices::default())
        } else {
            None
        }
    }

    fn len(&self) -> usize { 1 }
}

impl std::fmt::Display for Phone<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for &Phone<'_> {
    fn default() -> Self {
        &Phone::Bound
    }
}

/// Builds a list of phones from an input
/// where each phone is a character or escaped character
/// and reformats whitespace as word bounderies
#[must_use]
pub fn build_phone_list(input: EscapedStr<'_>) -> Vec<Phone<'_>> {
    let input = input.inner();
    let mut substring = Substring::new(input);
    let mut phones = Vec::new();

    while let Some(c) = substring.peek() {
        substring.grow();

        match c {
            ESCAPE_CHAR => (),
            '\n' => {
                _ = substring.pass();
                if !phones.last().is_some_and(Phone::is_bound) {
                    phones.push(Phone::Bound);
                }
                phones.push(Phone::Symbol("\n"));
                phones.push(Phone::Bound);
            },
            _ if c.is_whitespace() => {
                _ = substring.pass();
                if !phones.last().is_some_and(Phone::is_bound) {
                    phones.push(Phone::Bound);
                }
            },
            _ => phones.push(Phone::Symbol(substring.pass())),
        }
    }

    if !substring.str().is_empty() {
        phones.push(Phone::Symbol(substring.str()));
    }

    phones
}

/// Converts a list of string slices to a string
/// reformating word bounderies as whitespace
#[must_use]
pub fn phone_list_to_string(phone_list: &[Phone]) -> String {
    // creates a string
    let s = phone_list.iter()
        .map(Phone::as_str)
        .collect::<String>();

    // splits on escape characters
    let mut v = s.split(ESCAPE_CHAR).collect::<Vec<_>>();

    // removes empty strings on the ends of `v`
    v.pop_if(|s| s.is_empty());
    let slice = if let Some(&"") = v.first() {
        &v[1..]
    } else {
        &v[..]
    };

    // reinserts escaped escaped chars
    slice.iter()
        .map(|s| if s.is_empty() {
            const {
                // makes a string out of `ESCAPE_CHAR`
                let utf8 = std::ptr::from_ref(&ESCAPE_CHAR)
                    .cast::<[u8; ESCAPE_CHAR.len_utf8()]>();
                unsafe {
                    str::from_utf8_unchecked(&*utf8)
                }
            }
        } else {
            s
        })
        .collect()
}