use std::{fmt::{Debug, Display}, marker::PhantomData};

use crate::{applier::ApplicationError, keywords::{ANY_CHAR, ARG_SEP_CHAR, GAP_STR}, matcher::{choices::{Choices, OwnedChoices}, match_state::MatchState, Phones}, phones::Phone, rules::tokens::{RuleToken, ScopeId}, tokens::{Direction, ScopeType}};

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
            Self::Phone(phone) => MatchState::matches(phone, phones, choices),
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

impl Display for Pattern<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Phone(phone) => write!(f, "{}", phone.match_state.as_symbol()),
            Self::NonBound(any) => write!(f, "{}", any.match_state),
            Self::Gap(gap) => write!(f, "{gap}"),
            Self::Optional(option) => write!(f, "{option}"),
            Self::Selection(selection) => write!(f, "{selection}"),
        }
    }
}

/// A `MatchState` wrapper that lets `next_match` only be called once before exausting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CheckBox<'r, 's: 'r, T: MatchState<'r, 's>> {
    checked: bool,
    match_state: T,
    _phantom_lifetime_r: PhantomData<&'r ()>,
    _phantom_lifetime_s: PhantomData<&'s ()>,
}

impl<'r, 's: 'r, T: MatchState<'r, 's>> CheckBox<'r, 's, T> {
    /// Creates a new `CheckBox` wrapper
    const fn new(match_state: T) -> Self {
        Self {
            checked: false,
            match_state,
            _phantom_lifetime_r: PhantomData,
            _phantom_lifetime_s: PhantomData,
        }
    }
}

impl<'r, 's: 'r, T: MatchState<'r, 's>> MatchState<'r, 's> for CheckBox<'r, 's, T> {
    fn matches(&self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        self.match_state.matches(phones, choices)
    }

    fn next_match(&mut self, phones: &Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        if self.checked {
            None
        } else {
            self.checked = true;
            self.match_state.next_match(phones, choices)
        }
    }

    fn len(&self) -> usize {
        self.match_state.len()
    }

    fn reset(&mut self) {
        self.checked = false;
        self.match_state.reset();
    }
}

impl<'r, 's: 'r> MatchState<'r, 's> for Phone<'s> {
    fn matches(&self, phones: &mut Phones<'_, 's>, _: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        let matches = Phone::matches(self, phones.next());

        if matches {
            Some(OwnedChoices::default())
        } else {
            None
        }
    }

    fn next_match(&mut self, phones: &Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        MatchState::matches(self, &mut phones.clone(), choices)
    }

    fn reset(&mut self) {}

    fn len(&self) -> usize { 1 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NonBound<'r, 's> {
    id: Option<&'r ScopeId<'s>>,
}

impl<'r, 's: 'r> MatchState<'r, 's> for NonBound<'r, 's> {
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

    fn next_match(&mut self, phones: &Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        self.matches(&mut phones.clone(), choices)
    }

    fn reset(&mut self) {}

    fn len(&self) -> usize { 1 }
}

impl Display for NonBound<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = self.id {
            write!(f, "{id}")?;
        }

        write!(f, "{ANY_CHAR}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Gap<'s> {
    len: usize,
    checked_at_zero: bool,
    id: Option<&'s str>,
}

impl<'r, 's: 'r> MatchState<'r, 's> for Gap<'s> {
    fn matches(&self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        for _ in 0..self.len() {
            if phones.next().is_bound() {
                // returns `None` if a bound is crossed
                return None;
            }
        }

        let mut new_choices = choices.partial_clone();

        if let Some(id) = self.id {
            if let Some(max_len) = choices.gap.get(id).copied() {
                if self.len > max_len {
                    // if the max len is exceeded the match fails and the gap should exaust
                    return None;
                }
            } else {
                // sets the choice if it is the first gap with the id
                new_choices.gap.to_mut().insert(id, self.len);
            }
        }

        Some(new_choices.owned_choices())
    }

    fn next_match(&mut self, phones: &Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        if self.checked_at_zero {
            self.len += 1;
        } else {
            self.checked_at_zero = true;
        }
        
        self.matches(&mut phones.clone(), choices)
    }

    fn len(&self) -> usize {
        self.len
    }

    fn reset(&mut self) {
        self.len = 0;
        self.checked_at_zero = false;
    }
}

impl Display for Gap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = self.id {
            write!(f, "{id}")?;
        }

        write!(f, " {GAP_STR} ")
    }
}

/// A list of matchable `Pattern`s
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PatternList<'r, 's> {
    checked_at_initial: bool,
    patterns: Vec<Pattern<'r, 's>>,
}

impl<'r, 's> From<&'r [RuleToken<'s>]> for PatternList<'r, 's> {
    fn from(tokens: &'r [RuleToken<'s>]) -> Self {
        let patterns = tokens.iter()
            .map(Pattern::from)
            .collect();

        Self { patterns, checked_at_initial: false }
    }
}

impl<'r, 's> PatternList<'r, 's> {
    /// Creates a new `PatternList`
    pub const fn new(patterns: Vec<Pattern<'r, 's>>) -> Self {
        Self { patterns, checked_at_initial: false }
    }

    /// Determines if there are no patterns in the list
    pub const fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Converts a list of patterns to phones
    pub fn as_phones(&self, choices: &Choices<'_, 'r, 's>) -> Result<Vec<Phone<'s>>, ApplicationError<'r, 's>> {
        let mut phones = Vec::new();

        for pattern in &self.patterns {
            match pattern {
                Pattern::Phone(CheckBox { match_state: phone, .. }) => phones.push(*phone),

                Pattern::NonBound(CheckBox { match_state: NonBound { id: Some(id) }, ..}) =>
                if let Some(phone) = choices.any.get(id) {
                    phones.push(*phone);
                } else {
                    return Err(ApplicationError::PatternCannotBeConvertedToPhones(pattern.clone()));
                },

                Pattern::Gap(Gap { id: Some(id), .. }) =>
                match choices.gap.get(id) {
                    Some(0) => (),
                    _ => return Err(ApplicationError::PatternCannotBeConvertedToPhones(pattern.clone())),
                }
                
                Pattern::Optional(Optional { id: Some(id), option, .. }) =>
                if let Some(selected) = choices.optional.get(id).copied() {
                    if selected {
                        phones.append(&mut option.as_phones(choices)?);
                    }
                } else {
                    return Err(ApplicationError::PatternCannotBeConvertedToPhones(pattern.clone()));
                },

                Pattern::Selection(Selection { id: Some(id), options, .. }) => 
                if let Some(choice) = choices.selection.get(id).copied() 
                && let Some(option) = options.get(choice) {
                    phones.append(&mut option.as_phones(choices)?);
                } else {
                    return Err(ApplicationError::PatternCannotBeConvertedToPhones(pattern.clone()));
                },

                _ => return Err(ApplicationError::PatternCannotBeConvertedToPhones(pattern.clone())),
            }
        }

        Ok(phones)
    }

    // Recursively determines the next match of a sublist of the `PatternList` 
    fn next_sub_match(&mut self, index: usize, phones: &Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        let real_index = match phones.direction {
            _ if index >= self.patterns.len() => return Some(OwnedChoices::default()),
            Direction::Ltr => Some(index),
            Direction::Rtl => Some(self.patterns.len() - 1 - index),
        }?;

        loop {
            let mut new_choices = choices.partial_clone();
            let pat = &mut self.patterns[real_index];

            // finds the pattern's next match
            let pat_choices = pat.next_match(phones, &new_choices)?;
            new_choices.take_owned(pat_choices);

            // creates the phones for the remaining patterns
            let mut next_phones = phones.clone();
            next_phones.skip(pat.len());

            if let Some(next_choices) = self.next_sub_match(index + 1, &next_phones, &new_choices) {
                // if the remaining patterns match there is another match
                new_choices.take_owned(next_choices);
            } else {
                match phones.direction {
                    Direction::Ltr => self.patterns.get_mut(real_index + 1..).unwrap_or_default(),
                    Direction::Rtl => &mut self.patterns[..real_index]
                }.iter_mut().for_each(MatchState::reset);

                continue;
            }

            return Some(new_choices.owned_choices())
        }
    }
}

impl<'r, 's: 'r> MatchState<'r, 's> for PatternList<'r, 's> {
    fn matches(&self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        let mut new_choices = choices.partial_clone();

        // matches each pattern and saves the choices
        // if a pattern fails to match, the list fails to match
        match phones.direction() {
            Direction::Ltr => for pat in &self.patterns {
                let pattern_choices = pat.matches(phones, &new_choices)?;
                new_choices.take_owned(pattern_choices);
            },
            Direction::Rtl => for pat in self.patterns.iter().rev() {
                let pattern_choices = pat.matches(phones, &new_choices)?;
                new_choices.take_owned(pattern_choices);
            },
        }

        Some(new_choices.owned_choices())
    }

    fn next_match(&mut self, phones: &Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        if !self.checked_at_initial {
            self.checked_at_initial = true;
            if let Some(new_choices) = self.matches(&mut phones.clone(), choices) {
                return Some(new_choices);
            }
        }

        self.next_sub_match(0, phones, choices)
    }

    fn len(&self) -> usize {
        self.patterns.iter().fold(0, |acc, pat| acc + pat.len())
    }

    fn reset(&mut self) {
        self.patterns.iter_mut().for_each(MatchState::reset);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Optional<'r, 's> {
    selected: bool,
    option: PatternList<'r, 's>,
    id: Option<&'r ScopeId<'s>>,
}

impl<'r, 's: 'r> MatchState<'r, 's> for Optional<'r, 's> {
    fn matches(&self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        if let Some(id) = self.id {
            if let Some(choice) = choices.optional.get(id).copied() {
                // if choice and selection do not align, the match fails
                if self.selected != choice {
                    return None;
                }

                // checks the match if the choice aligns with the selection
                if choice {
                    // checks if the option matches
                    self.option.matches(phones, choices)
                } else {
                    // if the option is not inserted the pattern matches
                    Some(OwnedChoices::default())
                }
            } else {
                // chooses the selection and checks it
                let mut new_choices = choices.partial_clone();
                new_choices.optional.to_mut().insert(id, self.selected);

                // checks if the option matches with the new selection
                if self.selected {
                    let internal_choices = self.option.matches(phones, &new_choices)?;
                    new_choices.take_owned(internal_choices);
                }

                Some(new_choices.owned_choices())
            }
        } else if self.selected {
            // checks if the option matches
            self.option.matches(phones, choices)
        } else {
            // if the option is not inserted the pattern matches
            Some(OwnedChoices::default())
        }
    }

    fn next_match(&mut self, phones: &Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        if self.selected {
            loop {
                if self.option.next_match(phones, choices).is_some() {
                    if let Some(new_choices) = self.matches(&mut phones.clone(), choices) {
                        return Some(new_choices);
                    }

                    continue;
                }

                break;
            }

            self.selected = false;
            self.option.reset();
        } else {
            return None
        }
            
        self.matches(&mut phones.clone(), choices)
    }

    fn len(&self) -> usize {
        if self.selected {
            self.option.len()
        } else{
            0
        }
    }

    fn reset(&mut self) {
        self.selected = true;
        self.option.reset();
    }
}

impl Display for Optional<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = self.id {
            write!(f, "{id}")?;
        }

        let s = self.option.patterns.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");

        write!(f, "{} {s} {}", ScopeType::Optional.fmt_start(), ScopeType::Optional.fmt_end())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection<'r, 's> {
    /// should always contain at least one item
    options: Vec<PatternList<'r, 's>>,
    selected_index: usize,
    id: Option<&'r ScopeId<'s>>,
}

impl<'r, 's: 'r> MatchState<'r, 's>  for Selection<'r, 's> {
    fn matches(&self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        let mut new_choices = choices.partial_clone();

        if let Some(id) = self.id {
            if let Some(choice) = choices.selection.get(id).copied() {
                if self.selected_index != choice {
                    // selections cannot be changed
                    return None;
                }

                // checks if the choice matches
                let option = self.options.get(choice)?;
                let internal_choices = option.matches(phones, &new_choices)?;

                new_choices.take_owned(internal_choices);
            } else {
                // chooses the current index
                let option = self.options.get(self.selected_index)?;

                new_choices.selection.to_mut().insert(id, self.selected_index);

                let internal_choices = option.matches(phones, &new_choices)?;

                new_choices.take_owned(internal_choices);
            }
        } else {
            // checks if the selection matches
            let option = self.options.get(self.selected_index)?;
            let internal_choices = option.matches(phones, &new_choices)?;

            new_choices.take_owned(internal_choices);
        }

        Some(new_choices.owned_choices())
    }

    fn next_match(&mut self, phones: &Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        loop {
            // checks if the option has a next match form
            if self.options.get_mut(self.selected_index)?.next_match(phones, choices).is_some() {
                // checks if the pattern matches
                if let Some(new_choices) = self.matches(&mut phones.clone(), choices) {
                    return Some(new_choices);
                }
                
                continue;
            }
            
            // if there is not another match, moves to the next option
            self.selected_index += 1;

            // if the next match is invalid, the match fails
            if self.selected_index >= self.options.len() {
                return None;
            }
        }
    }

    fn len(&self) -> usize {
        self.options.get(self.selected_index).map(MatchState::len).unwrap_or_default()
    }

    fn reset(&mut self) {
        self.selected_index = 0;
        self.options.iter_mut().for_each(MatchState::reset);
    }
}

impl Display for Selection<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = self.id {
            write!(f, "{id}")?;
        }

        let s = self.options.iter()
            .map(|option|
                option.patterns.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            )
            .collect::<Vec<_>>()
            .join(&format!("{ARG_SEP_CHAR} "));

        write!(f, "{} {s} {}", ScopeType::Selection.fmt_start(), ScopeType::Selection.fmt_end())
    }
}