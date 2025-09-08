use crate::{applier::ApplicationError, matcher::{choices::{Choices, OwnedChoices}, match_state::MatchState, pattern::PatternList, Phones}, rules::{conditions::{AndType, Cond, CondType}, tokens::RuleToken}, tokens::Direction};

/// A matchable pattern for a rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulePattern<'r, 's> {
    input: PatternList<'r, 's>,
    conds: Vec<CondPattern<'r, 's>>,
    anti_conds: Vec<CondPattern<'r, 's>>,
}

fn contains_gap(tokens: &[RuleToken]) -> bool {
    for token in tokens {
        match token {
            RuleToken::Gap { .. } => return true,
            RuleToken::OptionalScope { content, .. } if contains_gap(content) => return true,
            RuleToken::SelectionScope { options, .. } if options.iter().any(|tokens| contains_gap(tokens)) => return true,
            _ => (),
        }
    }

    false
}

impl<'r, 's: 'r> RulePattern<'r, 's> {
    pub fn new(input: &'r [RuleToken<'s>], conds: &'r [Cond<'s>], anti_conds: &'r [Cond<'s>]) -> Result<Self, ApplicationError<'r, 's>> {
        if contains_gap(input) {
            return Err(ApplicationError::GapOutOfCond);
        }

        Ok(Self {
            input: input.into(),
            conds: conds.iter().map(CondPattern::from).collect(),
            anti_conds: anti_conds.iter().map(CondPattern::from).collect(),
        })
    }
    
    pub fn next_match(&mut self, phones: &Phones<'_, 's>) -> Result<Option<OwnedChoices<'r, 's>>, ApplicationError<'r, 's>> {
        let mut new_choices = Choices::default();

        loop {
            // checks the input
            let Some(input_choices) = self.input.next_match(phones, &new_choices) else {
                return Ok(None);
            };
            self.conds.iter_mut().for_each(CondPattern::reset);
            self.anti_conds.iter_mut().for_each(CondPattern::reset);

            new_choices.take_owned(input_choices);

            // prepares to create condition phones
            let mut after_input_phones = phones.clone();
            after_input_phones.skip(self.input.len());

            // creates the phone iterators for the conditions
            let cond_phones = match phones.direction {
                Direction::Ltr => CondPhoneInput {
                    left: phones.rtl_from_left(),
                    right: after_input_phones,
                },
                Direction::Rtl => CondPhoneInput {
                    left: after_input_phones,
                    right: phones.ltr_from_right(),
                },
            };

            // checks each condition agains each anti-condition
            for cond in &mut self.conds {
                // checks each match of each condition agains each anti-condition
                'cond_loop: while let Some(cond_choices) = cond.next_match(&cond_phones, &new_choices)? {
                    let mut post_cond_choices = new_choices.partial_clone();
                    post_cond_choices.take_owned(cond_choices.clone());

                    // checks agains each anti-condition
                    for anti_cond in &mut self.anti_conds {
                        // if an anti-condition matches, checks the next match of the condition
                        if anti_cond.next_match(&cond_phones, &post_cond_choices)?.is_some() {
                            anti_cond.reset();
                            continue 'cond_loop;
                        }

                        anti_cond.reset();
                    }
                    new_choices.take_owned(post_cond_choices.owned_choices());
                    cond.reset();
                    return Ok(Some(new_choices.owned_choices()));
                }

                cond.reset();
            }
        }
    }

    pub fn len(&self) -> usize {
        self.input.len()
    }
}

/// Both sides of the input phones to be matched by conditions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CondPhoneInput<'p, 's> {
    left: Phones<'p, 's>,
    right: Phones<'p, 's>,
}

/// A matchable pattern for a condition or anti-condition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CondPattern<'r, 's> {
    left: PatternList<'r, 's>,
    right: PatternList<'r, 's>,
    cond_type: CondType,
    and: Option<(AndType, Box<Self>)>,
    checked_once: bool,
}

impl<'r, 's> From<&'r Cond<'s>> for CondPattern<'r, 's> {
    fn from(cond: &'r Cond<'s>) -> Self {
        CondPattern {
            left: cond.left().into(),
            right: cond.right().into(),
            cond_type: cond.kind(),
            and: cond.and().map(|(and_type, and_cond)| (and_type, Box::new(and_cond.into()))),
            checked_once: false,
        }
    }
}

impl<'r, 's> CondPattern<'r, 's> {
    fn next_match(&mut self, phones: &CondPhoneInput<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Result<Option<OwnedChoices<'r, 's>>, ApplicationError<'r, 's>> {
        let mut new_choices = choices.partial_clone();

        // if all sides are empty and the condition has already been checked,
        // the condition is exausted
        if self.checked_once && self.left.is_empty() && self.right.is_empty() {
            return Ok(None);
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
                            if self.left.is_empty() {
                                return Ok(None);
                            }
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
                            if self.right.is_empty() {
                                return Ok(None);
                            }
                            continue 'right_check;
                        },
                    }
                }

                self.checked_once = true;
                return Ok(Some(new_choices.owned_choices()));
            }
        }
    }

    fn reset(&mut self) {
        self.left.reset();
        self.right.reset();
        self.checked_once = false;

        if let Some((_, and_cond)) = &mut self.and {
            and_cond.as_mut().reset();
        }
    }
}