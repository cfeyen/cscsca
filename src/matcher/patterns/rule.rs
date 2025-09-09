use crate::{applier::ApplicationError, matcher::{choices::{Choices, OwnedChoices}, match_state::MatchState, patterns::{cond::{CondPattern, CondPhoneInput}, list::PatternList}, phones::Phones}, rules::{conditions::Cond, tokens::RuleToken}, tokens::Direction};

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
            let cond_phones = match phones.direction() {
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