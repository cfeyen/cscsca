use std::marker::PhantomData;

use crate::matcher::{choices::{Choices, OwnedChoices}, match_state::{MatchState, UnitState}, phones::Phones};

/// A `MatchState` complient wrapper for a `UnitState`
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct CheckBox<'s, T: UnitState<'s>> {
    pub(super) checked: bool,
    pub unit_state: T,
    _phantom_lifetime_s: PhantomData<&'s ()>,
}

impl<'s, T: UnitState<'s>> CheckBox<'s, T> {
    /// Creates a new `CheckBox` wrapper
    pub const fn new(match_state: T) -> Self {
        Self {
            checked: false,
            unit_state: match_state,
            _phantom_lifetime_s: PhantomData,
        }
    }
}

impl<'s, T: UnitState<'s>> MatchState<'s> for CheckBox<'s, T> {
    fn matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        self.unit_state.matches(phones, choices)
    }

    fn next_match<'p>(&mut self, phones: &Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        if self.checked {
            None
        } else {
            self.checked = true;
            self.unit_state.matches(&mut phones.clone(), choices)
        }
    }

    fn len(&self) -> usize {
        self.unit_state.len()
    }
    fn max_len(&self) -> usize {
        self.unit_state.max_len()
    }

    fn reset(&mut self) {
        self.checked = false;
    }
}

impl<'s, T: UnitState<'s> + std::fmt::Debug> std::fmt::Debug for CheckBox<'s, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckBox")
            .field("checked", &self.checked)
            .field("unit_state", &self.unit_state)
            .finish()
    }
}