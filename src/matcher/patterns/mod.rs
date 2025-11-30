use std::cell::RefCell;

use crate::{
    matcher::{
        choices::{Choices, OwnedChoices},
        match_state::MatchState,
        patterns::{check_box::CheckBox, repetition::Repetition, list::PatternList, non_bound::NonBound, optional::Optional, selection::Selection},
        phones::Phones
    },
    phones::Phone,
    tokens::ScopeId,
};

pub mod list;
pub mod cond;
pub mod rule;
pub mod non_bound;
pub mod repetition;
pub mod optional;
pub mod selection;
pub mod check_box;
pub mod ir_to_patterns;

#[cfg(test)]
mod tests;


/// A state machine that can be matched to phones of a specific pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern<'s> {
    Phone(CheckBox<'s, Phone<'s>>),
    NonBound(CheckBox<'s, NonBound<'s>>),
    Repetition(Repetition<'s>),
    Optional(Optional<'s>),
    Selection(Selection<'s>),
    List(PatternList<'s>),
}

impl<'s> Pattern<'s> {
    pub const fn new_phone(phone: Phone<'s>) -> Self {
        Self::Phone(CheckBox::new(phone))
    }

    pub const fn new_any(id: Option<ScopeId<'s>>) -> Self {
        Self::NonBound(CheckBox::new(NonBound { id }))
    }

    pub fn new_repetition(id: Option<&'s str>, inclusive: PatternList<'s>, exclusive: Option<PatternList<'s>>) -> Self {
        Self::Repetition(Repetition {
            checked_at_zero: false,
            inclusive, exclusive: exclusive.map(RefCell::new),
            included: PatternList::default(),
            inclusions: 0,
            len: 0,
            id,
        })
    }

    pub const fn new_optional(content: Vec<Pattern<'s>>, id: Option<ScopeId<'s>>) -> Self {
        Self::Optional(Optional {
            selected: true,
            option: PatternList::new(content),
            id
        })
    }

    pub fn new_selection(options: Vec<Vec<Pattern<'s>>>, id: Option<ScopeId<'s>>) -> Self {
        let options = options.into_iter()
            .map(PatternList::new)
            .collect::<Vec<_>>();

        Self::Selection(Selection {
            options: if options.is_empty() {
                vec![PatternList::default()]
            } else {
                options
            },
            selected_index: 0,
            id
        })
    }
}

impl<'s> MatchState<'s> for Pattern<'s> {
    fn matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        match self {
            Self::Phone(phone) => phone.matches(phones, choices),
            Self::NonBound(any) => any.matches(phones, choices),
            Self::Repetition(repetition) => repetition.matches(phones, choices),
            Self::Optional(option) => option.matches(phones, choices),
            Self::Selection(selection) => selection.matches(phones, choices),
            Self::List(list) => list.matches(phones, choices),
        }
    }

    fn next_match<'p>(&mut self, phones: &Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        match self {
            Self::Phone(phone) => phone.next_match(phones, choices),
            Self::NonBound(any) => any.next_match(phones, choices),
            Self::Repetition(repetition) => repetition.next_match(phones, choices),
            Self::Optional(option) => option.next_match(phones, choices),
            Self::Selection(selection) => selection.next_match(phones, choices),
            Self::List(list) => list.next_match(phones, choices),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Phone(phone) => phone.len(),
            Self::NonBound(any) => any.len(),
            Self::Repetition(repetition) => repetition.len(),
            Self::Optional(option) => option.len(),
            Self::Selection(selection) => selection.len(),
            Self::List(list) => list.len(),
        }
    }

    fn max_len(&self) -> usize {
        match self {
            Self::Phone(phone) => phone.max_len(),
            Self::NonBound(any) => any.max_len(),
            Self::Repetition(repetition) => repetition.max_len(),
            Self::Optional(option) => option.max_len(),
            Self::Selection(selection) => selection.max_len(),
            Self::List(list) => list.max_len(),
        }
    }

    fn reset(&mut self) {
        match self {
            Self::Phone(phone) => phone.reset(),
            Self::NonBound(any) => any.reset(),
            Self::Repetition(repetition) => repetition.reset(),
            Self::Optional(option) => option.reset(),
            Self::Selection(selection) => selection.reset(),
            Self::List(list) => list.reset(),
        }
    }
}

impl std::fmt::Display for Pattern<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Phone(phone) => write!(f, "{}", phone.unit_state.as_symbol()),
            Self::NonBound(any) => write!(f, "{}", any.unit_state),
            Self::Repetition(repetition) => write!(f, "{repetition}"),
            Self::Optional(option) => write!(f, "{option}"),
            Self::Selection(selection) => write!(f, "{selection}"),
            Self::List(list) => write!(f, "{list}"),
        }
    }
}