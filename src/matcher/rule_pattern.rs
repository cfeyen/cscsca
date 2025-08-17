use crate::{matcher::{choices::{Choices, OwnedChoices}, match_state::{AdvanceResult, MatchState, PhoneInput}, pattern::PatternList, Phones}, phones::Phone, rules::{conditions::{AndType, Cond, CondType}, tokens::RuleToken}, tokens::Direction};

/// A part of a rule
/// 
/// Used to optimize `RulePattern` advancement
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
enum MatchSection {
    Input,
    Cond,
    #[default]
    AntiCond,
}

/// A matchable pattern for a rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulePattern<'r, 's> {
    input: PatternList<'r, 's>,
    conds: Vec<CondPattern<'r, 's>>,
    anti_conds: Vec<CondPattern<'r, 's>>,
    /// used to optimize advances,
    last_match_failure: MatchSection,
}

impl<'r, 's: 'r> RulePattern<'r, 's> {
    pub fn new(input: &'r [RuleToken<'s>], conds: &'r [Cond<'s>], anti_conds: &'r [Cond<'s>]) -> Self {
        Self {
            input: input.into(),
            conds: conds.iter().map(CondPattern::from).collect(),
            anti_conds: anti_conds.iter().map(CondPattern::from).collect(),
            last_match_failure: MatchSection::AntiCond,
        }
    }
}

impl<'p, 'r, 's: 'r + 'p> MatchState<'p, 'r, 's> for RulePattern<'r, 's> {
    type PhoneInput = Phones<'p, 's>;

    fn advance(&mut self, choices: &Choices<'_, 'r, 's>, direction: Direction) -> AdvanceResult {
        if self.last_match_failure == MatchSection::Input {
            // if the input did not match in the last match,
            // resets all condition/anti-conditions and advance the input
            self.conds.iter_mut().for_each(MatchState::reset);
            self.anti_conds.iter_mut().for_each(MatchState::reset);
            return self.input.advance(choices, direction);
        }
        
        if self.last_match_failure == MatchSection::AntiCond {
            // advances the last anti-condition,
            // if it is exausted, resets it and advances the next,
            // and so forth, so on
            for anti_cond in &mut self.anti_conds {
                match anti_cond.advance(choices, direction) {
                    AdvanceResult::Advanced => return AdvanceResult::Advanced,
                    AdvanceResult::Exausted => anti_cond.reset(),
                }
            }
        } else {
            self.anti_conds.iter_mut().for_each(MatchState::reset);
        }

        // if all anti-conditions were exausted or conds where where the last match faild
        // advances the last condition,
        // if it is exausted, resets it and advances the next,
        // and so forth, so on
        for cond in &mut self.conds {
            match cond.advance(choices, direction) {
                AdvanceResult::Advanced => return AdvanceResult::Advanced,
                AdvanceResult::Exausted => cond.reset(),
            }
        }

        // if all anti-conditions were exausted
        // advances the input,
        self.input.advance(choices, direction)

    }

    fn matches(&mut self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        let initial_phones = phones.clone();
        let mut new_choices = choices.partial_clone();

        // checks if the input matches
        let Some(input_choices) = self.input.matches(phones, &new_choices) else {
            self.last_match_failure = MatchSection::Input;
            return None;
        };
        new_choices.take_owned(input_choices);

        // creates the phone iterators for the conditions
        let cond_phones = match phones.direction {
            Direction::Ltr => CondPhoneInput {
                left: initial_phones.rtl_from_left(),
                right: phones.clone(),
            },
            Direction::Rtl => CondPhoneInput {
                left: phones.clone(),
                right: initial_phones.ltr_from_right(),
            },
        };

        self.last_match_failure = MatchSection::Cond;

        // checks each condition against each anti-condition
        // if one passes, its choices are returned
        'cond_loop: for cond in &mut self.conds {
            if let Some(cond_internal_choices) = cond.matches(&mut cond_phones.clone(), &new_choices) {
                let mut cond_choices = new_choices.partial_clone();
                cond_choices.take_owned(cond_internal_choices);

                for anti_cond in &mut self.anti_conds {
                    // if an anti-condition matches the cond fails
                    // and the next is checked
                    if anti_cond.next_match(&cond_phones, &cond_choices).is_some() {
                        self.last_match_failure = MatchSection::Cond;
                        continue 'cond_loop;
                    }
                }

                // if no anti-conditions match return the successful choices
                new_choices.take_owned(cond_choices.owned_choices());
                return Some(new_choices.owned_choices());
            }
        }

        // if no conditions pass, the match fails
        None
    }

    fn len(&self) -> usize {
        self.input.len()
    }

    fn reset(&mut self) {
        self.input.reset();
        self.conds.iter_mut().for_each(MatchState::reset);
        self.anti_conds.iter_mut().for_each(MatchState::reset);
        self.last_match_failure = MatchSection::default();
    }
}

/// Both sides of the input phones to be matched by conditions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CondPhoneInput<'p, 's> {
    left: Phones<'p, 's>,
    right: Phones<'p, 's>,
}

impl<'p, 's: 'p> PhoneInput<'p, 's> for CondPhoneInput<'p, 's> {
    fn next(&mut self) -> &'p Phone<'s> {
        <&Phone>::default()
    }

    fn direction(&self) -> Direction {
        Direction::Ltr
    }
}

/// A matchable pattern for a condition or anti-condition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CondPattern<'r, 's> {
    left: PatternList<'r, 's>,
    right: PatternList<'r, 's>,
    cond_type: CondType,
    and: Option<(AndType, Box<Self>)>
}

impl<'r, 's> From<&'r Cond<'s>> for CondPattern<'r, 's> {
    fn from(cond: &'r Cond<'s>) -> Self {
        CondPattern {
            left: cond.left().into(),
            right: cond.right().into(),
            cond_type: cond.kind(),
            and: cond.and().map(|(and_type, and_cond)| (and_type, Box::new(and_cond.into()))),
        }
    }
}

impl<'p, 'r, 's: 'r + 'p> MatchState<'p, 'r, 's> for CondPattern<'r, 's> {
    type PhoneInput = CondPhoneInput<'p, 's>;

    fn advance(&mut self, choices: &Choices<'_, 'r, 's>, _: Direction) -> AdvanceResult {
        // Advances the last and condition
        if let Some((_, and_cond)) = &mut self.and {
            let and_cond = and_cond.as_mut();

            match and_cond.advance(choices, Direction::Ltr) {
                AdvanceResult::Advanced => return AdvanceResult::Advanced,
                // resets the and condition before advanceing the proceeding
                AdvanceResult::Exausted => and_cond.reset(),
            }
        }

        // if there are no and conditions or they are exauseted,
        // advances the right
        match self.right.advance(choices, Direction::Ltr) {
            AdvanceResult::Advanced => return AdvanceResult::Advanced,
            // resets the right before advancing the left
            AdvanceResult::Exausted => self.right.reset(),
        }

        // if the right cannot be advanced, the left is advanced
        // unless it is a match condition, then advancement isn't needed
        if self.cond_type == CondType::Pattern {
            match self.left.advance(choices, Direction::Ltr) {
                AdvanceResult::Advanced => return AdvanceResult::Advanced,
                AdvanceResult::Exausted => self.left.reset(),
            }
        }

        // if nothing advances, the condition is exaused
        AdvanceResult::Exausted
    }

    fn matches(&mut self, phones: &mut CondPhoneInput<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        let CondPhoneInput {
            left: mut left_phones,
            right: mut right_phones,
        } = phones.clone();

        let mut new_choices = choices.partial_clone();

        match self.cond_type {
            // ensures patterns match enviroment
            CondType::Pattern => {
                let right_choices = self.right.matches(&mut right_phones, &new_choices)?;
                new_choices.take_owned(right_choices);

                let left_choices = self.left.matches(&mut left_phones, &new_choices)?;
                new_choices.take_owned(left_choices);
            },
            // ensures both sides match
            CondType::Match => {
                // creates phones from the left
                let left_phones = self.left.as_phones(&new_choices)?;
                let mut left_phones = Phones::new(&left_phones, 0, Direction::Ltr);

                // checks if the right matches the left
                let right_choices = self.right.matches(&mut left_phones, &new_choices)?;
                new_choices.take_owned(right_choices);
            },
        }

        if let Some((and_type, and_cond)) = &mut self.and {
            let and_cond = and_cond.as_mut();

            // checks the and condition
            let and_match = and_cond.matches(phones, choices);

            // ensures the and condition match is correct
            match (and_type, and_match) {
                (AndType::And, Some(and_choices)) => new_choices.take_owned(and_choices),
                (AndType::AndNot, None) => (),
                _ => return None,
            }
        }

        Some(new_choices.owned_choices())
    }

    fn len(&self) -> usize {
        0
    }

    fn reset(&mut self) {
        self.left.reset();
        self.right.reset();

        if let Some((_, and_cond)) = &mut self.and {
            and_cond.as_mut().reset();
        }
    }
}