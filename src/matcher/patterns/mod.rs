use crate::{matcher::{choices::{Choices, OwnedChoices}, match_state::MatchState, patterns::{check_box::CheckBox, gap::Gap, list::PatternList, non_bound::NonBound, optional::Optional, selection::Selection}, phones::Phones}, phones::Phone, rules::tokens::{RuleToken, ScopeId}};

mod list;
mod cond;
pub mod rule;
mod non_bound;
mod gap;
mod optional;
mod selection;
mod check_box;

#[cfg(test)]
mod tests;


/// A state machine that can be matched to phones of a specific pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern<'r, 's> {
    Phone(CheckBox<'r, 's, Phone<'s>>),
    NonBound(CheckBox<'r, 's, NonBound<'r, 's>>),
    Gap(Gap<'s>),
    Optional(Optional<'r, 's>),
    Selection(Selection<'r, 's>),
}

impl<'r, 's> From<&'r RuleToken<'s>> for Pattern<'r, 's> {
    fn from(token: &'r RuleToken<'s>) -> Self {
        match token {
            RuleToken::Phone(phone) => Self::new_phone(*phone),
            RuleToken::Any { id } => Self::new_any(id.as_ref()),
            RuleToken::Gap { id } => Self::new_gap(*id),
            RuleToken::OptionalScope { id, content } => Self::new_optional(
                content.iter().map(Self::from).collect(),
                id.as_ref(),
            ),
            RuleToken::SelectionScope { id, options } => Self::new_selection(
                options.iter().map(|tokens| tokens.iter().map(Self::from).collect()).collect(),
                id.as_ref(),
            ),
        }
    }
}

impl<'r, 's> Pattern<'r, 's> {
    pub const fn new_phone(phone: Phone<'s>) -> Self {
        Self::Phone(CheckBox::new(phone))
    }

    pub const fn new_any(id: Option<&'r ScopeId<'s>>) -> Self {
        Self::NonBound(CheckBox::new(NonBound { id }))
    }

    pub const fn new_gap(id: Option<&'s str>) -> Self {
        Self::Gap(Gap { len: 0, checked_at_zero: false, id })
    }

    pub const fn new_optional(content: Vec<Pattern<'r, 's>>, id: Option<&'r ScopeId<'s>>) -> Self {
        Self::Optional(Optional {
            selected: true,
            option: PatternList::new(content),
            id
        })
    }

    pub fn new_selection(options: Vec<Vec<Pattern<'r, 's>>>, id: Option<&'r ScopeId<'s>>) -> Self {
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

impl<'r, 's: 'r> MatchState<'r, 's> for Pattern<'r, 's> {
    fn matches(&self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        match self {
            Self::Phone(phone) => phone.matches(phones, choices),
            Self::NonBound(any) => any.matches(phones, choices),
            Self::Gap(gap) => gap.matches(phones, choices),
            Self::Optional(option) => option.matches(phones, choices),
            Self::Selection(selection) => selection.matches(phones, choices),
        }
    }

    fn next_match(&mut self, phones: &Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        match self {
            Self::Phone(phone) => phone.next_match(phones, choices),
            Self::NonBound(any) => any.next_match(phones, choices),
            Self::Gap(gap) => gap.next_match(phones, choices),
            Self::Optional(option) => option.next_match(phones, choices),
            Self::Selection(selection) => selection.next_match(phones, choices),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Phone(phone) => phone.len(),
            Self::NonBound(any) => any.len(),
            Self::Gap(gap) => gap.len(),
            Self::Optional(option) => option.len(),
            Self::Selection(selection) => selection.len(),
        }
    }

    fn reset(&mut self) {
        match self {
            Self::Phone(phone) => phone.reset(),
            Self::NonBound(any) => any.reset(),
            Self::Gap(gap) => gap.reset(),
            Self::Optional(option) => option.reset(),
            Self::Selection(selection) => selection.reset(),
        }
    }
}

impl std::fmt::Display for Pattern<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Phone(phone) => write!(f, "{}", phone.unit_state.as_symbol()),
            Self::NonBound(any) => write!(f, "{}", any.unit_state),
            Self::Gap(gap) => write!(f, "{gap}"),
            Self::Optional(option) => write!(f, "{option}"),
            Self::Selection(selection) => write!(f, "{selection}"),
        }
    }
}