use std::cell::RefCell;

use crate::{keywords::NOT_CHAR, matcher::{choices::{Choices, OwnedChoices}, match_state::MatchState, patterns::Pattern, phones::Phones}};

/// A pattern that represents a pattern but not a different pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Negative<'s> {
    pattern: Pattern<'s>,
    negative_pattern: RefCell<Pattern<'s>>,
}

impl<'s> Negative<'s> {
    pub const fn new(pattern: Pattern<'s>, negative_pattern: Pattern<'s>) -> Self {
        Self {
            pattern,
            negative_pattern: RefCell::new(negative_pattern),
        }
    }

    fn negative_matches<'p>(&self, phones: &Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> bool where 's: 'p {
        let mut negative_pattern = self.negative_pattern.borrow_mut();
        
        negative_pattern.reset();
        negative_pattern.next_match(phones, choices).is_some()
    }
}

impl<'s> MatchState<'s> for Negative<'s> {
    fn next_match<'p>(&mut self, phones: &Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        while let Some(new_choices) = self.pattern.next_match(phones, choices) {
            if self.negative_matches(phones, choices) {
                continue;
            }
            
            return Some(new_choices);
        }

        None
    }

    fn matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        if self.negative_matches(phones, choices) {
            None
        } else {
            self.pattern.matches(phones, choices)
        }
    }

    fn reset(&mut self) {
        self.pattern.reset();
    }

    fn len(&self) -> usize {
        self.pattern.len()
    }

    fn advance_once(&mut self) {
        self.pattern.advance_once();
    }
}

impl std::fmt::Display for Negative<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {NOT_CHAR} {}", self.pattern, self.negative_pattern.borrow())
    }
}