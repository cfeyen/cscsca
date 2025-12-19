use crate::{
    keywords::ANY_CHAR,
    matcher::{choices::{Choices, OwnedChoices}, match_state::UnitState, phones::Phones},
    tokens::ScopeId,
};

/// A pattern that represents a non-boundary phone
/// 
/// Should be used in a `CheckBox`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NonBound<'s> {
    pub id: Option<ScopeId<'s>>,
}

impl<'s> UnitState<'s> for NonBound<'s> {
    fn matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        let phone = phones.next();
        let mut new_choices = choices.partial_clone();

        if let Some(id) = &self.id {
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
                new_choices.any.to_mut().insert(id.clone(), *phone);
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

impl std::fmt::Display for NonBound<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = &self.id {
            write!(f, "{id}")?;
        }

        write!(f, "{ANY_CHAR}")
    }
}