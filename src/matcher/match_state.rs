use crate::{matcher::{choices::{Choices, OwnedChoices}, phones::Phones}};

/// A state machine that determines whether or not a rule should be applied
pub trait MatchState<'r, 's: 'r> {
    /// Determines if a state matches phones
    fn matches(&self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>>;

    /// Resets to a default state
    fn reset(&mut self);

    /// gets the number of phones in the state
    fn len(&self) -> usize;

    /// Advances a state to the next valid match and returns the choices made to get there
    /// 
    /// If there is no remaining valid match, `None` is returned
    fn next_match(&mut self, phones: &Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>>;
}

/// A signle-state varient of `MatchState`
pub trait UnitState<'r, 's: 'r> {
    /// Determines if a state matches phones
    fn matches(&self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>>;

    /// gets the number of phones in the state
    fn len(&self) -> usize;
}