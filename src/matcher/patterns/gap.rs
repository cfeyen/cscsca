use crate::{keywords::GAP_STR, matcher::{choices::{Choices, OwnedChoices}, match_state::MatchState, phones::Phones}};

/// A pattern that represents some non-negative number (possibly zero) of non-boundary phones
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Gap<'s> {
    pub(super) len: usize,
    pub(super) checked_at_zero: bool,
    pub(super) id: Option<&'s str>,
}

impl<'r, 's: 'r> MatchState<'r, 's> for Gap<'s> {
    fn matches(&self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        for _ in 0..self.len() {
            if phones.next().is_bound() {
                // returns `None` if a bound is crossed
                return None;
            }
        }

        let mut new_choices = choices.partial_clone();

        if let Some(id) = self.id {
            if let Some(max_len) = choices.gap.get(id).copied() {
                if self.len > max_len {
                    // if the max len is exceeded the match fails and the gap should exaust
                    return None;
                }
            } else {
                // sets the choice if it is the first gap with the id
                new_choices.gap.to_mut().insert(id, self.len);
            }
        }

        Some(new_choices.owned_choices())
    }

    fn next_match(&mut self, phones: &Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        if self.checked_at_zero {
            self.len += 1;
        } else {
            self.checked_at_zero = true;
        }
        
        self.matches(&mut phones.clone(), choices)
    }

    fn len(&self) -> usize {
        self.len
    }

    fn reset(&mut self) {
        self.len = 0;
        self.checked_at_zero = false;
    }
}

impl std::fmt::Display for Gap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = self.id {
            write!(f, "{id}")?;
        }

        write!(f, " {GAP_STR} ")
    }
}