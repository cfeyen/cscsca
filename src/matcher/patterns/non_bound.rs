use crate::{keywords::ANY_CHAR, matcher::{choices::{Choices, OwnedChoices}, match_state::UnitState, phones::Phones}, rules::tokens::ScopeId};

/// A pattern that represents a non-boundary phone
/// 
/// Should be used in a `CheckBox`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NonBound<'r, 's> {
    pub(super) id: Option<&'r ScopeId<'s>>,
}

impl<'r, 's: 'r> UnitState<'r, 's> for NonBound<'r, 's> {
    fn matches(&self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        let phone = phones.next();
        let mut new_choices = choices.partial_clone();

        if let Some(id) = self.id {
            if let Some(choice) = new_choices.any.get(id) {
                // if the phone matches the choice the pattern matches,
                // otherwise it doesn't
                if phone.matches(choice) {
                    Some(new_choices.owned_choices())
                } else {
                    None
                }
            } else if !phone.is_bound() {
                // if the phone isn't a bound the choice is made
                new_choices.any.to_mut().insert(id, *phone);
                Some(new_choices.owned_choices())
            } else {
                None
            }
        } else if !phone.is_bound() {
            // matches if the phone isn't a bound
            Some(new_choices.owned_choices())
        } else {
            None
        }
    }

    fn len(&self) -> usize { 1 }
}

impl std::fmt::Display for NonBound<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = self.id {
            write!(f, "{id}")?;
        }

        write!(f, "{ANY_CHAR}")
    }
}