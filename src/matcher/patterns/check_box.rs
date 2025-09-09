use std::marker::PhantomData;

use crate::matcher::{choices::{Choices, OwnedChoices}, match_state::{MatchState, UnitState}, phones::Phones};

/// A `MatchState` complient wrapper for a `UnitState`
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct CheckBox<'r, 's: 'r, T: UnitState<'r, 's>> {
    pub(super) checked: bool,
    pub(super) unit_state: T,
    _phantom_lifetime_r: PhantomData<&'r ()>,
    _phantom_lifetime_s: PhantomData<&'s ()>,
}

impl<'r, 's: 'r, T: UnitState<'r, 's>> CheckBox<'r, 's, T> {
    /// Creates a new `CheckBox` wrapper
    pub const fn new(match_state: T) -> Self {
        Self {
            checked: false,
            unit_state: match_state,
            _phantom_lifetime_r: PhantomData,
            _phantom_lifetime_s: PhantomData,
        }
    }
}

impl<'r, 's: 'r, T: UnitState<'r, 's>> MatchState<'r, 's> for CheckBox<'r, 's, T> {
    fn matches(&self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        self.unit_state.matches(phones, choices)
    }

    fn next_match(&mut self, phones: &Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
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

    fn reset(&mut self) {
        self.checked = false;
    }
}

impl<'r, 's, T: UnitState<'r, 's> + std::fmt::Debug> std::fmt::Debug for CheckBox<'r, 's, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckBox")
            .field("checked", &self.checked)
            .field("unit_state", &self.unit_state)
            .finish()
    }
}