use std::{cell::RefCell, fmt::Write as _};

use crate::{
    applier::ApplicationError,
    ir::tokens::{Break, IrToken},
    matcher::{
        choices::{Choices, OwnedChoices},
        match_state::MatchState,
        patterns::{
            cond::{CondPattern, CondPhoneInput},
            ir_to_patterns::RuleStructureError,
            list::PatternList,
            optional::Optional,
            selection::Selection,
            Pattern,
        },
        phones::Phones,
    },
    tokens::{Direction, Shift}
};

/// A matchable pattern for a rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulePattern<'s> {
    input: PatternList<'s>,
    conds: Vec<CondPattern<'s>>,
    anti_conds: Vec<CondPattern<'s>>,
}

fn contains_gap(tokens: &PatternList<'_>) -> bool {
    for token in tokens.inner() {
        match token {
            Pattern::Gap { .. } => return true,
            Pattern::Optional(Optional { option, ..}) if contains_gap(option) => return true,
            Pattern::Selection(Selection { options, .. }) if options.iter().any(|tokens| contains_gap(tokens)) => return true,
            _ => (),
        }
    }

    false
}

impl<'s> RulePattern<'s> {
    pub fn new(input: PatternList<'s>, mut conds: Vec<CondPattern<'s>>, anti_conds: Vec<CondPattern<'s>>) -> Result<Self, RuleStructureError<'s>> {
        if contains_gap(&input) {
            return Err(RuleStructureError::GapOutOfCond);
        }

        if conds.is_empty() {
            conds = vec![CondPattern::default()];
        }

        Ok(Self {
            input,
            conds,
            anti_conds,
        })
    }

    pub const fn input(&self) -> &PatternList<'s> {
        &self.input
    }

    pub fn conds(&self) -> &[CondPattern<'s>] {
        &self.conds
    }

    pub fn anti_conds(&self) -> &[CondPattern<'s>] {
        &self.anti_conds
    }
    
    pub fn next_match<'p>(&mut self, phones: &Phones<'_, 'p>) -> Result<Option<OwnedChoices<'p>>, ApplicationError<'s>> where 's: 'p {
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
            let mut after_input_phones = *phones;
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

    pub fn reset(&mut self) {
        self.input.reset();
        self.conds.iter_mut().for_each(CondPattern::reset);
        self.anti_conds.iter_mut().for_each(CondPattern::reset);
    }
}

/// A collection of data that define a sound change rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundChangeRule<'s> {
    pub kind: Shift,
    /// The tokens that represent what should replace the input
    pub output: Vec<Pattern<'s>>,
    pub pattern: RefCell<RulePattern<'s>>,
}

impl std::fmt::Display for SoundChangeRule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let input = self.pattern.borrow()
            .input()
            .inner()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");

        let output = self.output
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");

        let mut conds = String::new();
        for cond in self.pattern.borrow().conds() {
            _ = write!(conds, " {} {cond}", IrToken::Break(Break::Cond));
        }

        let mut anti_conds = String::new();
        for anti_cond in self.pattern.borrow().anti_conds() {
            _ = write!(anti_conds, " {} {anti_cond}", IrToken::Break(Break::AntiCond));
        }
        
        write!(f, "{} {} {}{}{}", input, &self.kind, output, conds, anti_conds)
    }
}