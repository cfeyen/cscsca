use std::cell::RefCell;

use crate::{keywords::{GAP_END_CHAR, GAP_START_CHAR, NOT_CHAR}, matcher::{choices::{Choices, OwnedChoices}, match_state::MatchState, patterns::{Pattern, list::PatternList}, phones::Phones}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gap<'s> {
    pub(super) checked_at_zero: bool,
    pub(super) inclusive: PatternList<'s>,
    pub(super) exclusive: Option<RefCell<PatternList<'s>>>,
    pub(super) included: PatternList<'s>,
    pub(super) len: usize,
    pub(super) id: Option<&'s str>,
}

impl<'s> MatchState<'s> for Gap<'s> {
    fn matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        if let Some(mut exclusive) = self.exclusive.as_ref().map(RefCell::borrow_mut) {
            let mut phones2 = *phones;

            let len = self.id.as_ref()
                .and_then(|id| choices.gap.get(id))
                .copied()
                .unwrap_or(self.len);
            
            for _ in 0..len {
                if exclusive.next_match(&phones2, choices).is_some() {
                    exclusive.reset();
                    return None;
                }

                phones2.next();
            }

            exclusive.reset();
        }

        if self.included.len() == self.len && let Some(new_choices) = self.included.matches(phones, choices) {
            Some(new_choices)
        } else {
            None
        }
    }

    fn next_match<'p>(&mut self, phones: &Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        if self.checked_at_zero || self.id.as_ref().map(|id| choices.gap.contains_key(id)).is_some_and(|exists| exists) {
            let mut new_choices = choices.partial_clone();

            let mut max_len = phones.rem_len();

            if let Some(id) = &self.id && let Some(max) = choices.gap.get(id).copied() {
                max_len = max.max(max_len);
            }

            loop {
                loop {
                    if let Some(indcluded_choices) = self.included.next_match(phones, &new_choices) {
                        let mut choices = new_choices.partial_clone();
                        choices.take_owned(indcluded_choices);

                        if let Some(match_choices) = self.matches(&mut phones.clone(), &choices) {
                            choices.take_owned(match_choices);

                            if let Some(id) = &self.id && !choices.gap.contains_key(id) {
                                choices.gap.to_mut().insert(id, self.len);
                            }

                            new_choices.take_owned(choices.owned_choices());

                            return Some(new_choices.owned_choices());
                        }
                    } else {
                        if self.included.inner().len() > max_len {
                            break;
                        }
                        
                        self.included.push(Pattern::List(self.inclusive.clone()));
                    }
                }

                if self.len > max_len {
                    break;
                }

                self.len += 1;
                self.included = PatternList::default();
            }

            None
        } else {
            self.checked_at_zero = true;
            self.len = 0;
            self.included = PatternList::default();

            if let Some(id) = self.id {
                let mut new_choices = choices.partial_clone();
                new_choices.gap.to_mut().insert(id, self.len);
                
                Some(new_choices.owned_choices())
            } else {
                Some(OwnedChoices::default())
            }
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn reset(&mut self) {
        self.checked_at_zero = false;
        self.len = 0;
        self.included = PatternList::default();
    }
}

impl std::fmt::Display for Gap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = &self.id {
            write!(f, "{id}")?;
        }

        write!(f, "{GAP_START_CHAR} {} ", self.inclusive)?;

        if let Some(exclusive) = &self.exclusive {
            write!(f, "{NOT_CHAR} {} ", exclusive.borrow())?;
        }

        write!(f, "{GAP_END_CHAR}")
    }
}