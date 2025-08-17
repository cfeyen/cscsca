use crate::{matcher::choices::{Choices, OwnedChoices}, phones::Phone, tokens::Direction};

/// The result of advancing a `MatchState`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvanceResult {
    Advanced,
    Exausted,
}

/// An `Iterator`-like type that is used to check a `MatchState`
pub trait PhoneInput<'p, 's: 'p> {
    fn next(&mut self) -> &'p Phone<'s>;

    fn direction(&self) -> Direction;
}

/// A state machine that determines whether or not a rule should be applied
pub trait MatchState<'p, 'r, 's: 'r + 'p> {
    type PhoneInput: PhoneInput<'p, 's> + Clone;

    /// Advances to the next state
    fn advance(&mut self, choices: &Choices<'_, 'r, 's>, direction: Direction) -> AdvanceResult;

    /// Determines if a state matches phones
    /// 
    /// `self` should only be mutated to match choices
    fn matches(&mut self, phones: &mut Self::PhoneInput, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>>;

    /// Resets to a default state
    fn reset(&mut self);

    /// gets the number of phones in the state
    fn len(&self) -> usize;

    /// Advances a state to the next valid match and returns the choices made to get there
    /// 
    /// If there is no remaining valid match, `None` is returned
    fn next_match(&mut self, phones: &Self::PhoneInput, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        loop {
            let mut phones = phones.clone();

            // if the state is a match, returns the choices made
            if let Some(new_choices) = self.matches(&mut phones, choices) {
                return Some(new_choices)
            }

            // advances to the next state,
            // returns `None` if there is no next state
            if self.advance(choices, phones.direction()) == AdvanceResult::Exausted {
                return None
            }
        }
    }
}