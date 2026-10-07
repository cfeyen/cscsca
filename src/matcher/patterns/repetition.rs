use crate::{
    keywords::{ARG_SEP_CHAR, MATCH_CHAR, REPETITION_END_CHAR, REPETITION_START_CHAR}, matcher::{
        choices::{Choices, OwnedChoices},
        match_state::MatchState,
        patterns::{ir_to_patterns::RuleStructureError, list::PatternList},
        phones::Phones
    }, tokens::{Direction, RepetitionNumber, ScopeId},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repetition<'s> {
    pattern: PatternList<'s>,
    included: PatternList<'s>,
    inclusions: usize,
    pub id: Option<ScopeId<'s>>,
    len: usize,
    min: RepetitionNumber,
    max: Option<RepetitionNumber>,
}

impl<'s> MatchState<'s> for Repetition<'s> {
    fn matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        // gets the phones that the repetition must try to match
        let match_phones = {
            let mut phones = *phones;
            let mut match_phones = Vec::new();

            for _ in 0..self.len() {
                match_phones.push(*phones.next());
            }

            // make the match phones ltr
            if phones.direction() == Direction::Rtl {
                match_phones.reverse();
            }

            match_phones
        };
        
        let mut new_choices = choices.partial_clone();
        
        if let Some(id) = &self.id {
            if let Some(target_phones) = new_choices.repetition.get(id) {
                // if the target phones don't match, the match must fail
                if &match_phones != target_phones {
                    return None;
                }
            } else {
                new_choices.repetition.to_mut().insert(id.clone(), match_phones);
            }
        }

        if
            self.len >= self.min as usize
            && self.len <= self.max_len(phones)
            && self.len == self.included.len()
            && let Some(included_choices) = self.included.matches(&mut phones.clone(), &new_choices)
        {
            new_choices.take_owned(included_choices);
            Some(new_choices.owned_choices())
        } else {
            None
        }
    }

    fn next_match<'p>(&mut self, phones: &Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        if self.min as usize > self.len {
            self.len = self.min as _;
        }

        let mut new_choices = choices.partial_clone();

        let max_len = self.max_len(phones);

        // checks each varient up to the maximum length 
        loop {
            if let Some(included_choices) = self.included.next_match(phones, &new_choices) {
                let mut choices = new_choices.partial_clone();
                choices.take_owned(included_choices);

                if let Some(match_choices) = self.matches(&mut phones.clone(), &choices) {
                    choices.take_owned(match_choices);
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
    }

    fn len(&self) -> usize {
        self.len
    }

    fn reset(&mut self) {
        self.included = PatternList::default();
        self.inclusions = 0;

        for _ in 0..self.min {
            self.add_inclusion();
        }

        self.len = 0;
    }

    fn advance_once(&mut self) {
        self.included.advance_once();
    }
}

impl<'s> Repetition<'s> {
    pub fn new(id: Option<ScopeId<'s>>, pattern: PatternList<'s>, min: RepetitionNumber, max: Option<RepetitionNumber>) -> Result<Self, RuleStructureError<'s>> {
        if let Some(max) = max && min > max {
            return Err(RuleStructureError::MinExceedsMax { min, max });
        }

        Ok(Self {
            pattern,
            included: PatternList::default(),
            inclusions: 0,
            len: 0,
            id,
            min,
            max,
        })
    }

    fn max_len(&self, phones: &Phones<'_, '_>) -> usize {
        let mut max_len = phones.rem_len();

        if let Some(max) = self.max {
            max_len = max_len.min(max as _);
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