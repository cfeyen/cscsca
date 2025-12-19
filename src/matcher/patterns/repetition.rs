use std::cell::RefCell;

use crate::{
    keywords::{REPETITION_END_CHAR, REPETITION_START_CHAR, NOT_CHAR},
    matcher::{choices::{Choices, OwnedChoices}, match_state::MatchState, patterns::list::PatternList, phones::Phones},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repetition<'s> {
    pub(super) checked_at_zero: bool,
    pub(super) inclusive: PatternList<'s>,
    pub(super) exclusive: Option<RefCell<PatternList<'s>>>,
    pub(super) included: PatternList<'s>,
    pub(super) inclusions: usize,
    pub(super) len: usize,
    pub(super) id: Option<&'s str>,
}

impl<'s> Repetition<'s> {
    fn get_max_len(&self, choices: &Choices<'_, '_>) -> Option<usize> {
        self.id.and_then(|id| choices.repetition.get(id)).copied()
    }

    fn exclusive_matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> bool where 's: 'p {
        let max_ex_len = self.get_max_len(choices).unwrap_or(self.len);

        if let Some(mut exclusive) = self.exclusive.as_ref().map(RefCell::borrow_mut) {
            for i in 0..self.len {
                while exclusive.next_match(phones, choices).is_some() {
                    if i + exclusive.len() <= max_ex_len {
                        exclusive.reset();
                        return true
                    }
                }
                
                exclusive.reset();
                _ = phones.next();
            }
        }

        return false;
    }

    fn included_matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        if self.included.len() == self.len && let Some(new_choices) = self.included.matches(phones, choices) {
            Some(new_choices)
        } else {
            None
        }
    }
}

impl<'s> MatchState<'s> for Repetition<'s> {
    fn matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        if self.exclusive_matches(phones, choices) {
            None
        } else {
            self.included_matches(phones, choices)
        }
    }

    fn next_match<'p>(&mut self, phones: &Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        if self.checked_at_zero || self.id.as_ref().map(|id| choices.repetition.contains_key(id)).is_some_and(|exists| exists) {
            let mut new_choices = choices.partial_clone();

            // gets the maximum length of the repetition
            let mut max_len = phones.rem_len();
            if let Some(id) = &self.id && let Some(max) = choices.repetition.get(id).copied() {
                max_len = max.min(max_len);
            }

            // checks each varient up to the maximum length 
            loop {
                if self.exclusive_matches(&mut phones.clone(), choices) {
                    return None;
                }

                if let Some(included_choices) = self.included.next_match(phones, &new_choices) {
                    let mut choices = new_choices.partial_clone();
                    choices.take_owned(included_choices);

                    if let Some(match_choices) = self.included_matches(&mut phones.clone(), &choices) {
                        choices.take_owned(match_choices);

                        if let Some(id) = &self.id && !choices.repetition.contains_key(id) {
                            choices.repetition.to_mut().insert(id, self.len);
                        }

                        new_choices.take_owned(choices.owned_choices());

                        return Some(new_choices.owned_choices());
                    }
                } else {
                    self.included.reset();
                    for pat in self.inclusive.inner() {
                        self.included.push(pat.clone());
                    }
                    self.inclusions += 1;

                    if self.inclusions > max_len {
                        self.len += 1;
                        self.included = PatternList::default();
                        self.inclusions = 0;

                        if self.len > max_len {
                            break;
                        }
                    }
                }
            }

            None
        } else {
            // checks with a length of zero
            self.checked_at_zero = true;
            self.len = 0;
            self.inclusions = 0;
            self.included = PatternList::default();

            if let Some(id) = self.id {
                let mut new_choices = choices.partial_clone();
                new_choices.repetition.to_mut().insert(id, self.len);
                
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
        self.inclusions = 0;
    }

    fn advance_once(&mut self) {
        if !self.checked_at_zero {
            self.checked_at_zero = true;
        }
    }
}

impl std::fmt::Display for Repetition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = &self.id {
            write!(f, "{id}")?;
        }

        write!(f, "{REPETITION_START_CHAR} {} ", self.inclusive)?;

        if let Some(exclusive) = &self.exclusive {
            write!(f, "{NOT_CHAR} {} ", exclusive.borrow())?;
        }

        write!(f, "{REPETITION_END_CHAR}")
    }
}