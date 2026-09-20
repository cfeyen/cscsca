use crate::{
    applier::ApplicationError,
    matcher::{
        choices::{Choices, OwnedChoices},
        match_state::MatchState,
        patterns::{Pattern, check_box::CheckBox, non_bound::NonBound, optional::Optional, repetition::Repetition, selection::Selection},
        phones::Phones
    }, phones::Phone, tokens::Direction,
};

/// A list of matchable `Pattern`s
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PatternList<'s> {
    checked_at_end: bool,
    patterns: Vec<Pattern<'s>>,
}

impl<'s> PatternList<'s> {
    /// Creates a new `PatternList`
    pub const fn new(patterns: Vec<Pattern<'s>>) -> Self {
        Self { patterns, checked_at_end: false }
    }

    /// Gets the inner list of `Pattern`s
    pub fn inner(&self) -> &[Pattern<'s>] {
        &self.patterns
    }

    /// Adds a pattern to the end of the list
    pub fn push(&mut self, pat: Pattern<'s>) {
        self.patterns.push(pat);
    }

    /// Sets the flag marking the list as checked at its current position to `false`
    pub const fn checked_flag_reset(&mut self) {
        self.checked_at_end = false;
    }

    /// Converts a list of patterns to phones
    pub fn as_phones<'p>(&self, choices: &Choices<'_, 'p>) -> Result<Vec<Phone<'p>>, ApplicationError<'s>> where 's: 'p {
        let mut phones = Vec::new();

        for pattern in &self.patterns {
            match pattern {
                Pattern::Phone(CheckBox { unit_state: phone, .. }) => phones.push(*phone),

                Pattern::NonBound(CheckBox { unit_state: NonBound { id: Some(id) }, ..}) =>
                if let Some(phone) = choices.any.get(id) {
                    phones.push(*phone);
                } else {
                    return Err(ApplicationError::PatternCannotBeConvertedToPhones(pattern.clone()));
                },

                Pattern::Repetition(Repetition { id: Some(id), .. }) =>
                match choices.repetition.get(id) {
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

    fn recursive_submatch<'p>(&mut self, index: usize, phones: &Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        if index >= self.patterns.len() {
            if self.checked_at_end {
                return None;
            }
            
            self.checked_at_end = true;
            return Some(OwnedChoices::default());
        }

        // gets the actual index from the input index based on direction
        // (`index` phones from the initial side)
        let real_index = match phones.direction() {
            Direction::Ltr => index,
            Direction::Rtl => self.patterns.len() - 1 - index,
        };

        loop {
            let pat = &mut self.patterns[real_index];

            // creates the phones for the remaining patterns
            let mut next_phones = *phones;
            next_phones.skip(pat.len());

            let mut new_choices = choices.partial_clone();

            if let Some(pat_choices) = pat.matches(&mut phones.clone(), choices) {
                new_choices.take_owned(pat_choices);

                if let Some(next_choices) = self.recursive_submatch(index + 1, &next_phones, &new_choices) {
                    // if the following patterns match, return the success
                    new_choices.take_owned(next_choices);

                    return Some(new_choices.owned_choices());
                }
            }

            // if the pattern does not match, or no match exists for the following patterns,
            // advance the pattern and reset the following patterns
            let pat = &mut self.patterns[real_index];

            pat.advance_once();

            // If another match does not exist, then this pattern cannot match
            pat.next_match(phones, choices)?;

            match phones.direction() {
                Direction::Ltr => self.patterns.get_mut(real_index + 1..).unwrap_or_default(),
                Direction::Rtl => &mut self.patterns[..real_index]
            }.iter_mut().for_each(MatchState::reset);

            self.checked_at_end = false;
        }
    }
}

impl<'s> MatchState<'s> for PatternList<'s> {
    fn matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
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

    fn next_match<'p>(&mut self, phones: &Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        self.recursive_submatch(0, phones, choices)
    }

    fn len(&self) -> usize {
        self.patterns.iter().fold(0, |len, pat| len + pat.len())
    }

    fn reset(&mut self) {
        self.checked_at_end = false;
        self.patterns.iter_mut().for_each(MatchState::reset);
    }

    fn advance_once(&mut self) {
        self.patterns.iter_mut().for_each(MatchState::advance_once);
    }
}

impl std::fmt::Display for PatternList<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content_fmt = self.patterns.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");

        write!(f, "{content_fmt}")
    }
}