use crate::{
    applier::ApplicationError,
    matcher::{
        choices::{Choices, OwnedChoices},
        match_state::MatchState,
        patterns::{check_box::CheckBox, repetition::Repetition, non_bound::NonBound, optional::Optional, selection::Selection, Pattern},
        phones::Phones
    },
    phones::Phone,
    tokens::Direction,
};



/// A list of matchable `Pattern`s
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PatternList<'s> {
    checked_at_initial: bool,
    patterns: Vec<Pattern<'s>>,
}

impl<'s> PatternList<'s> {
    /// Creates a new `PatternList`
    pub const fn new(patterns: Vec<Pattern<'s>>) -> Self {
        Self { patterns, checked_at_initial: false }
    }

    /// Gets the inner list of `Pattern`s
    pub fn inner(&self) -> &[Pattern<'s>] {
        &self.patterns
    }

    pub fn push(&mut self, pat: Pattern<'s>) {
        self.patterns.push(pat);
    }

    /// Sets the flag marking the list as checked at its current position to `false`
    pub const fn checked_flag_reset(&mut self) {
        self.checked_at_initial = false;
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

    // Recursively determines the next match of a sublist of the `PatternList` 
    fn next_sub_match<'p>(&mut self, index: usize, phones: &Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        if index >= self.patterns.len() {
            return Some(OwnedChoices::default());
        }

        // gets the actual index from the input index based on direction
        // (`index` phones from the initial side)
        let real_index = match phones.direction() {
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
            let mut next_phones = *phones;
            next_phones.skip(pat.len());

            if let Some(next_choices) = self.next_sub_match(index + 1, &next_phones, &new_choices) {
                // if the remaining patterns match there is another match
                new_choices.take_owned(next_choices);
            } else {
                // resets all the patterns directionally after the real index
                match phones.direction() {
                    Direction::Ltr => self.patterns.get_mut(real_index + 1..).unwrap_or_default(),
                    Direction::Rtl => &mut self.patterns[..real_index]
                }.iter_mut().for_each(MatchState::reset);

                continue;
            }

            return Some(new_choices.owned_choices());
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
        if !self.checked_at_initial {
            self.checked_at_initial = true;
            if let Some(new_choices) = self.matches(&mut phones.clone(), choices) {
                return Some(new_choices);
            }
        } else if self.patterns.is_empty() {
            return None;
        }

        self.next_sub_match(0, phones, choices)
    }

    fn len(&self) -> usize {
        self.patterns.iter().fold(0, |acc, pat| acc + pat.len())
    }

    fn reset(&mut self) {
        self.checked_at_initial = false;
        self.patterns.iter_mut().for_each(MatchState::reset);
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