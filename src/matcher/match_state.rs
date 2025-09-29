use crate::{matcher::{choices::{Choices, OwnedChoices}, phones::Phones}};

/// A state machine that determines whether or not a rule should be applied
pub trait MatchState<'s> {
    /// Determines if a state matches phones
    fn matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p;

    /// Resets to a default state
    fn reset(&mut self);

    /// gets the number of phones in the state
    fn len(&self) -> usize;

    /// Advances a state to the next valid match and returns the choices made to get there
    /// 
    /// If there is no remaining valid match, `None` is returned
    fn next_match<'p>(&mut self, phones: &Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p;
}

/// A signle-state varient of `MatchState`
pub trait UnitState<'s> {
    /// Determines if a state matches phones
    fn matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p;

    /// gets the number of phones in the state
    fn len(&self) -> usize;
}