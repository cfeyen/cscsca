use crate::{matcher::{choices::{Choices, OwnedChoices}, match_state::{AdvanceResult, MatchState, PhoneInput}, Phones}, phones::Phone, rules::tokens::{RuleToken, ScopeId}, tokens::Direction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern<'r, 's> {
    Phone(Phone<'s>),
    NonBound(NonBound<'r, 's>),
    Gap(Gap<'s>),
    Optional(Optional<'r, 's>),
    Selection(Selection<'r, 's>),
}

impl<'r, 's> From<&'r RuleToken<'s>> for Pattern<'r, 's> {
    fn from(token: &'r RuleToken<'s>) -> Self {
        match token {
            RuleToken::Phone(phone) => Self::Phone(*phone),
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
    pub const fn new_any(id: Option<&'r ScopeId<'s>>) -> Self {
        Self::NonBound(NonBound { id })
    }

    pub const fn new_gap(id: Option<&'s str>) -> Self {
        Self::Gap(Gap { len: 0, exaust_on_next_match: false, id })
    }

    pub const fn new_optional(content: Vec<Pattern<'r, 's>>, id: Option<&'r ScopeId<'s>>) -> Self {
        Self::Optional(Optional {
            selected: true,
            option: PatternList { patterns: content },
            id
        })
    }

    pub fn new_selection(options: Vec<Vec<Pattern<'r, 's>>>, id: Option<&'r ScopeId<'s>>) -> Self {
        let options = options.into_iter()
            .map(|patterns| PatternList { patterns })
            .collect::<Vec<_>>();

        Self::Selection(Selection {
            options: if options.is_empty() {
                vec![PatternList { patterns: Vec::new() }]
            } else {
                options
            },
            selected_index: 0,
            id
        })
    }
}

impl<'p, 'r, 's: 'r + 'p> MatchState<'p, 'r, 's> for Pattern<'r, 's> {
    type PhoneInput = Phones<'p, 's>;

    fn advance(&mut self, choices: &Choices<'_, 'r, 's>, direction: Direction) -> AdvanceResult {
        match self {
            Self::Phone(phone) => phone.advance(choices, direction),
            Self::NonBound(any) => any.advance(choices, direction),
            Self::Gap(gap) => gap.advance(choices, direction),
            Self::Optional(option) => option.advance(choices, direction),
            Self::Selection(selection) => selection.advance(choices, direction),
        }
    }

    fn matches(&mut self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        match self {
            Self::Phone(phone) => MatchState::matches(phone, phones, choices),
            Self::NonBound(any) => any.matches(phones, choices),
            Self::Gap(gap) => gap.matches(phones, choices),
            Self::Optional(option) => option.matches(phones, choices),
            Self::Selection(selection) => selection.matches(phones, choices),
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

impl<'p, 'r, 's: 'r + 'p> MatchState<'p, 'r, 's> for Phone<'s> {
    type PhoneInput = Phones<'p, 's>;

    fn advance(&mut self, _: &Choices<'_, '_, 's>, _: Direction) -> AdvanceResult {
        AdvanceResult::Exausted
    }

    fn matches(&mut self, phones: &mut Phones<'_, 's>, _: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        let matches = Phone::matches(self, PhoneInput::next(phones));

        if matches {
            Some(OwnedChoices::default())
        } else {
            None
        }
    }

    fn reset(&mut self) {}

    fn len(&self) -> usize { 1 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NonBound<'r, 's> {
    id: Option<&'r ScopeId<'s>>,
}

impl<'p, 'r, 's: 'r + 'p> MatchState<'p, 'r, 's> for NonBound<'r, 's> {
    type PhoneInput = Phones<'p, 's>;

    fn advance(&mut self, _: &Choices<'_, 'r, 's>, _: Direction) -> AdvanceResult {
        AdvanceResult::Exausted
    }

    fn matches(&mut self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        let phone = PhoneInput::next(phones);
        let mut new_choices = choices.partial_clone();

        if let Some(id) = self.id {
            if let Some(choice) = new_choices.any.get(id) {
                // if the phone matcehs the choice the pattern matches,
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

    fn reset(&mut self) {}

    fn len(&self) -> usize { 1 }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Gap<'s> {
    len: usize,
    exaust_on_next_match: bool,
    id: Option<&'s str>,
}

impl<'p, 'r, 's: 'r + 'p> MatchState<'p, 'r, 's> for Gap<'s> {
    type PhoneInput = Phones<'p, 's>;

    fn advance(&mut self, choices: &Choices<'_, 'r, 's>, _: Direction) -> AdvanceResult {
        if self.exaust_on_next_match {
            // if a bound is crossed, the gap is exausted
            AdvanceResult::Exausted
        } else if let Some(id) = self.id 
            && let Some(max_len) = choices.gap.get(id)
            && self.len >= *max_len
        {
            // if the maximum chosen length is to be exceeded,
            // the gap is exausted
            AdvanceResult::Exausted
        } else {
            // otherwise the gap advances
            self.len += 1;
            // assumes that, unless checked in the next match, the gap is exausted
            self.exaust_on_next_match = true;
            AdvanceResult::Advanced
        }
    }

    fn matches(&mut self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        for _ in 0..self.len() {
            if PhoneInput::next(phones).is_bound() {
                // marks if any bound is crossed
                // then returns `None`
                self.exaust_on_next_match = true;
                return None;
            }
        }

        // marks if no bounds are crossed
        self.exaust_on_next_match = false;

        let mut new_choices = choices.partial_clone();

        if let Some(id) = self.id && !choices.gap.contains_key(id) {
            // sets the choice if it is the first gap with the id
            new_choices.gap.to_mut().insert(id, self.len);
        }

        Some(new_choices.owned_choices())
    }

    fn len(&self) -> usize {
        self.len
    }

    fn reset(&mut self) {
        self.len = 0;
        self.exaust_on_next_match = false;
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternList<'r, 's> {
    patterns: Vec<Pattern<'r, 's>>,
}

impl<'r, 's> From<&'r [RuleToken<'s>]> for PatternList<'r, 's> {
    fn from(tokens: &'r [RuleToken<'s>]) -> Self {
        let patterns = tokens.iter()
            .map(Pattern::from)
            .collect();

        Self { patterns }
    }
}

impl<'r, 's> PatternList<'r, 's> {
    #[cfg(test)]
    pub fn new(patterns: Vec<Pattern<'r, 's>>) -> Self {
        Self { patterns }
    }

    pub fn as_phones(&self, choices: &Choices<'_, 'r, 's>) -> Option<Vec<Phone<'s>>> {
        let mut phones = Vec::new();

        for pattern in &self.patterns {
            match pattern {
                Pattern::Phone(phone) => phones.push(*phone),

                Pattern::NonBound(NonBound { id: Some(id) }) =>
                if let Some(phone) = choices.any.get(id) {
                    phones.push(*phone);
                } else {
                    return None;
                },

                Pattern::Gap(Gap { id: Some(id), .. }) =>
                match choices.gap.get(id) {
                    Some(0) => (),
                    _ => return None,
                }
                
                Pattern::Optional(Optional { id: Some(id), option, .. }) =>
                if let Some(selected) = choices.optional.get(id).copied() {
                    if selected {
                        phones.append(&mut option.as_phones(choices)?);
                    }
                } else {
                    return None;
                },

                Pattern::Selection(Selection { id: Some(id), options, .. }) => 
                if let Some(choice) = choices.selection.get(id).copied() 
                && let Some(option) = options.get(choice) {
                    phones.append(&mut option.as_phones(choices)?);
                } else {
                    return None
                },

                _ => return None,
            }
        }

        Some(phones)
    }
}

impl<'p, 'r, 's: 'r + 'p> MatchState<'p, 'r, 's> for PatternList<'r, 's> {
    type PhoneInput = Phones<'p, 's>;

    fn advance(&mut self, choices: &Choices<'_, 'r, 's>, direction: Direction) -> AdvanceResult {
        // advances from the end toward the start
        // if an advancement is made the next state is acheived and `Advanced` is returned
        // otherwise the pattern is reset and the next is advanced
        match direction {
            Direction::Ltr => for pat in self.patterns.iter_mut().rev() {
                match pat.advance(choices, direction) {
                    AdvanceResult::Advanced => return AdvanceResult::Advanced,
                    AdvanceResult::Exausted => pat.reset(),
                }
            },
            Direction::Rtl => for pat in &mut self.patterns {
                match pat.advance(choices, direction) {
                    AdvanceResult::Advanced => return AdvanceResult::Advanced,
                    AdvanceResult::Exausted => pat.reset(),
                }
            },
        }

        AdvanceResult::Exausted
    }

    fn matches(&mut self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        let mut new_choices = choices.partial_clone();

        // matches each pattern and saves the choices
        // if a pattern fails to match, the list fails to match
        match phones.direction() {
            Direction::Ltr => for pat in &mut self.patterns {
                let pattern_choices = pat.matches(phones, &new_choices)?;
                new_choices.take_owned(pattern_choices);
            },
            Direction::Rtl => for pat in self.patterns.iter_mut().rev() {
                let pattern_choices = pat.matches(phones, &new_choices)?;
                new_choices.take_owned(pattern_choices);
            },
    }

        Some(new_choices.owned_choices())
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

impl<'p, 'r, 's: 'r + 'p> MatchState<'p, 'r, 's> for Optional<'r, 's> {
    type PhoneInput = Phones<'p, 's>;

    fn advance(&mut self, choices: &Choices<'_, 'r, 's>, direction: Direction) -> AdvanceResult {
        if let Some(id) = self.id && let Some(choice) = choices.optional.get(id).copied() {
            // if the selection and choice are both deselected pattern is exausted,
            // if the choice is selected, then, if the option can be advanced, the pattern is advanced
            // otherwise the pattern is exausted
            if !self.selected && !choice {
                AdvanceResult::Exausted
            } else if choice {
                self.selected = true;

                if let AdvanceResult::Exausted = self.option.advance(choices, direction) {
                    return AdvanceResult::Exausted;
                }

                AdvanceResult::Advanced
            } else {
                AdvanceResult::Exausted
            }
        } else {
            // if the option is exaused, it is deselected 
            // if the option is deselected, the pattern is exausted
            // otherwise, the option is advanced
            if self.selected {
                if let AdvanceResult::Exausted = self.option.advance(choices, direction) {
                    self.selected = false;
                } else {
                    self.selected = true;
                }

                AdvanceResult::Advanced
            } else {
                AdvanceResult::Exausted
            }
        }
    }

    fn matches(&mut self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        if let Some(id) = self.id {
            let mut new_choices = choices.partial_clone();

            if let Some(choice) = choices.optional.get(id).copied() {
                if !self.selected && choice {
                    // Optionals may not be reselected
                    return None;
                }
                self.selected = choice;

                // if the content doesn't match when selected, returns `None`
                // otherwise updates `new_choices`
                if choice {
                    let internal_choices = self.option.matches(phones, &new_choices)?;
                    new_choices.take_owned(internal_choices);
                }

                // returns the made chocies
                Some(new_choices.owned_choices())
            } else {
                // sets the scope's selection
                // if the content doesn't match when selected, returns `None`
                if self.selected {
                    new_choices.optional.to_mut().insert(id, true);
                    let internal_choices = self.option.matches(phones, &new_choices)?;
                    new_choices.take_owned(internal_choices);
                } else {
                    new_choices.optional.to_mut().insert(id, false);
                }

                // returns the made chocies
                Some(new_choices.owned_choices())
            }
        } else if self.selected {
            self.option.matches(phones, choices)
        } else {
            Some(OwnedChoices::default())
        }
    }

    fn len(&self) -> usize {
        if self.selected {
            self.option.len()
        } else {
            0
        }
    }

    fn reset(&mut self) {
        self.selected = true;
        self.option.reset();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection<'r, 's> {
    /// should always contain at least one item
    options: Vec<PatternList<'r, 's>>,
    selected_index: usize,
    id: Option<&'r ScopeId<'s>>,
}

impl<'p, 'r, 's: 'r + 'p> MatchState<'p, 'r, 's>  for Selection<'r, 's> {
    type PhoneInput = Phones<'p, 's>;

    fn advance(&mut self, choices: &Choices<'_, 'r, 's>, direction: Direction) -> AdvanceResult {
        let choice_made = if let Some(id) = self.id && let Some(choice) = choices.selection.get(id).copied() {
            if choice > self.selected_index {
                return AdvanceResult::Exausted;
            }
            self.selected_index = choice;
            true
        } else {
            false
        };
        
        if let Some(option) = self.options.get_mut(self.selected_index) {
            match option.advance(choices, direction) {
                AdvanceResult::Advanced => AdvanceResult::Advanced,
                AdvanceResult::Exausted => {
                    // if the selection is enforced it cannot be advanced
                    if choice_made {
                        return AdvanceResult::Exausted;
                    }

                    // moves to the next selection
                    self.selected_index += 1;

                    if self.selected_index >= self.options.len() {
                        // if the next selection is not valid the pattern is exausted
                        AdvanceResult::Exausted
                    } else {
                        AdvanceResult::Advanced
                    }
                }
            }
        } else {
            // if the selection is not valid the pattern is exausted
            AdvanceResult::Exausted
        }
    }

    fn matches(&mut self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        let mut new_choices = choices.partial_clone();

        if let Some(id) = self.id {
            if let Some(choice) = choices.selection.get(id).copied() {
                if self.selected_index > choice {
                    // selections cannot be moved backward
                    return None;
                }
                self.selected_index = choice;

                let option = self.options.get_mut(choice)?;
                let internal_choices = option.matches(phones, &new_choices)?;

                new_choices.take_owned(internal_choices);
            } else {
                let option = self.options.get_mut(self.selected_index)?;

                new_choices.selection.to_mut().insert(id, self.selected_index);

                let internal_choices = option.matches(phones, &new_choices)?;

                new_choices.take_owned(internal_choices);
            }
        } else {
            let option = self.options.get_mut(self.selected_index)?;
            let internal_choices = option.matches(phones, &new_choices)?;

            new_choices.take_owned(internal_choices);
        }

        Some(new_choices.owned_choices())
    }

    fn len(&self) -> usize {
        self.options.get(self.selected_index).map(MatchState::len).unwrap_or_default()
    }

    fn reset(&mut self) {
        self.selected_index = 0;
        self.options.iter_mut().for_each(MatchState::reset);
    }
}