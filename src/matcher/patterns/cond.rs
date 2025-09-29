use crate::{
    applier::ApplicationError,
    matcher::{
        choices::{Choices, OwnedChoices},
        match_state::MatchState,
        patterns::list::PatternList,
        phones::Phones,
    },
    tokens::{Direction, AndType, CondType},
};



/// Both sides of the input phones to be matched by conditions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CondPhoneInput<'p, 's> {
    pub(super) left: Phones<'p, 's>,
    pub(super) right: Phones<'p, 's>,
}

/// A matchable pattern for a condition or anti-condition
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CondPattern<'s> {
    left: PatternList<'s>,
    right: PatternList<'s>,
    cond_type: CondType,
    and: Option<(AndType, Box<Self>)>,
}

impl<'s> CondPattern<'s> {
    pub fn new(cond_type: CondType, left: PatternList<'s>, right: PatternList<'s>) -> Self {
        Self {
            left,
            right,
            cond_type,
            and: None,
        }
    }

    pub fn add_and(&mut self, and_type: AndType, and: Self) {
        if let Some((_, and_cond)) = &mut self.and {
            and_cond.add_and(and_type, and);
        } else {
            self.and = Some((and_type, Box::new(and)));
        }
    }

    pub(super) fn next_match<'p>(&mut self, phones: &CondPhoneInput<'_, 'p>, choices: &Choices<'_, 'p>) -> Result<Option<OwnedChoices<'p>>, ApplicationError<'s>> where 's: 'p {
        let mut new_choices = choices.partial_clone();

        // resets the checked flag on the left of the input
        // so the right can be fully checked before it is advanced
        if self.cond_type == CondType::Pattern {
            self.left.checked_flag_reset();
        }
        
        'left_check: loop {
            if self.cond_type == CondType::Pattern {
                let Some(left_choices) = self.left.next_match(&phones.left, &new_choices) else {
                    return Ok(None);
                };
                new_choices.take_owned(left_choices);
            }

            'right_check: loop {
                match self.cond_type {
                    CondType::Pattern => {
                        let Some(right_choices) = self.right.next_match(&phones.right, &new_choices) else {
                            // if the right cannot match, resets and looks for another match on the left
                            self.right.reset();
                            
                            continue 'left_check;
                        };
                        new_choices.take_owned(right_choices);
                    },
                    CondType::Match => {
                        // creates phones from the left
                        let left_phones = self.left.as_phones(&new_choices)?;
                        let left_phones = Phones::new(&left_phones, 0, Direction::Ltr);

                        // checks if the right matches the left
                        let Some(right_choices) = self.right.next_match(&left_phones, &new_choices) else {
                            return Ok(None);
                        };

                        // checks that the left and the right not only match
                        // but are also the same length
                        if self.left.len() != self.right.len() {
                            continue 'right_check;
                        }

                        new_choices.take_owned(right_choices);
                    }
                }

                if let Some((and_type, and_cond)) = &mut self.and {
                    let and_type = *and_type;
                    let and_cond = and_cond.as_mut();

                    // checks the and condition
                    let and_match = and_cond.next_match(phones, &new_choices)?;

                    // ensures the and condition match is correct
                    match (and_type, and_match) {
                        (AndType::And, Some(and_choices)) => new_choices.take_owned(and_choices),
                        (AndType::AndNot, None) => (),
                        _ => {
                            and_cond.reset();
                            
                            continue 'right_check;
                        },
                    }
                }

                return Ok(Some(new_choices.owned_choices()));
            }
        }
    }

    pub(super) fn reset(&mut self) {
        self.left.reset();
        self.right.reset();

        if let Some((_, and_cond)) = &mut self.and {
            and_cond.as_mut().reset();
        }
    }
}

impl std::fmt::Display for CondPattern<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.left, self.cond_type, self.right)?;

        if let Some((and_type, and_cond)) = &self.and {
            write!(f, "{and_type} {and_cond}")?;
        }

        Ok(())
    }
}