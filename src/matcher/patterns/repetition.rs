use crate::{
    keywords::{ARG_SEP_CHAR, MATCH_CHAR, REPETITION_END_CHAR, REPETITION_START_CHAR},
    matcher::{
        choices::{Choices, OwnedChoices},
        match_state::MatchState,
        patterns::{ir_to_patterns::RuleStructureError, list::PatternList},
        phones::Phones
    },
    tokens::RepetitionNumber,
};

// todo: [ pat = n, m ]
// todo: ensure agreement occurs in repetition count

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repetition<'s> {
    checked_at_zero: bool,
    pattern: PatternList<'s>,
    included: PatternList<'s>,
    inclusions: usize,
    pub(super) id: Option<&'s str>,
    len: usize,
    min: RepetitionNumber,
    max: Option<RepetitionNumber>,
}

impl<'s> MatchState<'s> for Repetition<'s> {
    fn matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        if
            self.min as usize <= self.len
            && self.max_len(phones, choices) >= self.len
            && self.included.len() == self.len
            && let Some(new_choices) = self.included.matches(phones, choices)
        {
            Some(new_choices)
        } else {
            None
        }
    }

    fn next_match<'p>(&mut self, phones: &Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        if self.min as usize > self.len {
            self.checked_at_zero = true;
            self.len = self.min as _;
        }

        if self.checked_at_zero || self.id.as_ref().map(|id| choices.repetition.contains_key(id)).is_some_and(|exists| exists) {
            let mut new_choices = choices.partial_clone();

            let max_len = self.max_len(phones, choices);

            // checks each varient up to the maximum length 
            loop {
                if let Some(included_choices) = self.included.next_match(phones, &new_choices) {
                    let mut choices = new_choices.partial_clone();
                    choices.take_owned(included_choices);

                    if let Some(match_choices) = self.matches(&mut phones.clone(), &choices) {
                        choices.take_owned(match_choices);

                        if let Some(id) = &self.id && !choices.repetition.contains_key(id) {
                            choices.repetition.to_mut().insert(id, self.len);
                        }

                        new_choices.take_owned(choices.owned_choices());

                        return Some(new_choices.owned_choices());
                    }
                } else {
                    self.included.reset();
                    for pat in self.pattern.inner() {
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
        self.included = PatternList::default();
        self.inclusions = 0;

        for _ in 0..self.min {
            self.add_inclusion();
        }

        self.len = 0;
    }

    fn advance_once(&mut self) {
        if !self.checked_at_zero {
            self.checked_at_zero = true;
        }
    }
}

impl<'s> Repetition<'s> {
    pub fn new(id: Option<&'s str>, pattern: PatternList<'s>, min: RepetitionNumber, max: Option<RepetitionNumber>) -> Result<Self, RuleStructureError<'s>> {
        if let Some(max) = max && min > max {
            return Err(RuleStructureError::MinExceedsMax { min, max });
        }

        Ok(Self {
            checked_at_zero: false,
            pattern,
            included: PatternList::default(),
            inclusions: 0,
            len: 0,
            id,
            min,
            max,
        })
    }

    fn max_len(&self, phones: &Phones<'_, '_>, choices: &Choices<'_, '_>) -> usize {
        let mut max_len = phones.rem_len();

        if let Some(max) = self.max {
            max_len = max_len.min(max as _);
        }

        if let Some(id) = &self.id && let Some(max) = choices.repetition.get(id).copied() {
            max_len = max.min(max_len);
        }

        max_len
    }

    fn add_inclusion(&mut self) {
        for _ in 0..self.inclusions {
            for pat in self.pattern.inner() {
                self.included.push(pat.clone());
            }
        }

        self.inclusions += 1;
    }
}

impl std::fmt::Display for Repetition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = &self.id {
            write!(f, "{id}")?;
        }

        write!(f, "{REPETITION_START_CHAR} {} ", self.pattern)?;

        if self.min > 0 || self.max.is_some() {
            write!(f, "{MATCH_CHAR} ")?;

            if let Some(max) = self.max {
                write!(f, "{}{ARG_SEP_CHAR} {max}", self.min)?;
            } else {
                write!(f, "{}", self.min)?;
            }
        }

        write!(f, " {REPETITION_END_CHAR}")
    }
}