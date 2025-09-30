use std::time::Instant;

use crate::{
    executor::runtime::LineApplicationLimit,
    ir::tokens::IrToken,
    matcher::{
        choices::Choices,
        patterns::{check_box::CheckBox, non_bound::NonBound, optional::Optional, selection::Selection, Pattern},
        phones::Phones,
        patterns::{rule::SoundChangeRule, ir_to_patterns::RuleStructureError},
    },
    phones::Phone,
    tokens::{Direction, ShiftType}
};

#[cfg(test)]
mod tests;

/// The condition for a line application limit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub (crate) enum LimitCondition {
    /// End time
    Time(Instant),
    /// Application attempts
    Count {
        attempts: usize,
        max: usize,
    },
}

impl From<LineApplicationLimit> for LimitCondition {
    fn from(val: LineApplicationLimit) -> Self {
        match val {
            LineApplicationLimit::Time(time) => LimitCondition::Time(Instant::now() + time),
            LineApplicationLimit::Attempts(max) => LimitCondition::Count { attempts: 0, max },
        }
    }
}

impl LimitCondition {
    /// Checks if the limiting condition has been exceeded,
    /// moves `Count` varient closer to completion
    fn check(&mut self) -> bool {
        match self {
            Self::Time(time) => Instant::now() >= *time,
            Self::Count { attempts, max } if *attempts >= *max => true,
            Self::Count { attempts, max: _ } => {
                *attempts += 1;
                false
            }
        }
    }
}

/// Applies a rule to a list of phones within a time limit
pub fn apply<'s: 'p, 'p>(rule: &SoundChangeRule<'s>, phones: &mut Vec<Phone<'p>>, limit: Option<LineApplicationLimit>) -> Result<(), ApplicationError<'s>> {
    let dir = rule.kind.dir;
    let mut phone_index = dir.start_index(phones);
    let mut limit_condition: Option<LimitCondition> = limit.map(Into::into);
    
    while phone_index < phones.len() {
        if let Some((replace_len, input_len)) = apply_at(rule, phones, phone_index)? {
            phone_index = next_position(rule, input_len, replace_len, phone_index, phones);
        } else {
            phone_index = dir.change_by_one(phone_index);
        }

        // returns an error if the limit is exceeded
        // protects against infinite loops
        if let Some(limit_condition) = limit_condition.as_mut()
            && limit_condition.check()
        {
            return Err(ApplicationError::ExceededLimit(*limit_condition));
        }
    }

    Ok(())
}

fn next_position(rule: &SoundChangeRule, input_len: usize, replace_len: usize, phone_index: usize, phones: &[Phone]) -> usize {
    let dir = rule.kind.dir;
    match (dir, rule.kind.kind) {
        // prevents repeat application on zero-sized inputs
        _ if input_len == 0 && replace_len == 0 => dir.change_by_one(phone_index),
        (Direction::Ltr, ShiftType::Move) => dir.change_by(phone_index, replace_len),
            // ensures removing a phone does not take the phone index out of the phone list ending the rule early
        (Direction::Rtl, _) if phone_index >= phones.len() => phones.len().wrapping_sub(1),
        (Direction::Rtl, ShiftType::Move) => dir.change_by(phone_index, input_len),
        _ => phone_index,
    }
}

/// Applies a rule to a location in a list of phones if the input and conds match
/// 
/// Return: (the length of the output, the length of what it replaced)
fn apply_at<'s: 'p, 'p>(rule: &SoundChangeRule<'s>, phones: &mut Vec<Phone<'p>>, phone_index: usize) -> Result<Option<(usize, usize)>, ApplicationError<'s>> {
    let SoundChangeRule {
        kind,
        output,
        pattern,
    } = rule;
    
    pattern.borrow_mut().reset();

    let match_phones = Phones::new(phones, phone_index, kind.dir);

    let mut choices = Choices::default();

    if let Some(new_choices) = pattern.borrow_mut().next_match(&match_phones)? {
        choices.take_owned(new_choices);
    } else {
        return Ok(None);
    }

    let input_len = pattern.borrow().len();

    replace_input(phones, phone_index, input_len, output, &choices, kind.dir)
}

/// Replaces the slice `phones[index..input_len]` with the output as phones
/// 
/// Return: (the length of the output, the length of what it replaced)
fn replace_input<'s: 'p, 'p>(phones: &mut Vec<Phone<'p>>, index: usize, input_len: usize, output: &[Pattern<'s>], choices: &Choices<'_, 'p>, dir: Direction) -> Result<Option<(usize, usize)>, ApplicationError<'s>> {
    let mut shifted_phones = Vec::new();

    let phone_iter = &mut phones.iter();

    // adds the proceeding phones to the new phones
    let input_start = match dir {
        Direction::Ltr => index,
        Direction::Rtl => index + 1 - input_len,
    };

    for &phone in phone_iter.take(input_start) {
        shifted_phones.push(phone);
    }

    // discards the input
    _ = phone_iter.take(input_len).count(); // since take is lazy, the count causes it to be used

    let mut output_phones = Vec::new();

    // adds the output
    for phone in patterns_to_phones(output, choices)? {
        // prevents in-output bound doubling
        if output_phones.last().is_some_and(Phone::is_bound) && phone.is_bound() {
            continue;
        }
        
        output_phones.push(phone);
    }

    let mut output_len = output_phones.len();

    // prevents bound doubling at the start of the output
    while shifted_phones.last().is_none_or(Phone::is_bound) && output_phones.first().is_some_and(Phone::is_bound) {
        shifted_phones.pop();
        output_len -= 1;
    }

    shifted_phones.append(&mut output_phones);
    drop(output_phones);

    let mut after_output_phones = Vec::new();

    // adds the following phones
    for &phone in phone_iter {
        after_output_phones.push(phone);
    }

    // prevents bound doubling at the end of the output
    while shifted_phones.last().is_none_or(Phone::is_bound) && after_output_phones.first().is_some_and(Phone::is_bound) {
        shifted_phones.pop();

        if shifted_phones.is_empty() {
            break;
        }
    }

    shifted_phones.append(&mut after_output_phones);
    drop(after_output_phones);

    *phones = shifted_phones;

    Ok(Some((output_len, input_len)))
}

/// Converts patterns to the phones that they represent according to choices that have been made
fn patterns_to_phones<'s: 'p, 'p>(patterns: &[Pattern<'s>], choices: &Choices<'_, 'p>) -> Result<Vec<Phone<'p>>, ApplicationError<'s>> {
    let mut phones = Vec::new();

    for pattern in patterns {
        match pattern {
            Pattern::Phone(phone) => phones.push(phone.unit_state),
            Pattern::NonBound(CheckBox { unit_state: NonBound{ id: Some(id) }, .. }) => {
                if let Some(phone) = choices.any().get(id) {
                    phones.push(*phone);
                } else {
                    return Err(ApplicationError::UnmatchedTokenInOutput(pattern.clone()));
                }
            },
            Pattern::Optional(Optional { id: Some(id), option, .. }) => {
                if let Some(insert) = choices.optional().get(id) {
                    if *insert {
                        for phone in patterns_to_phones(option.inner(), choices)? {
                            phones.push(phone);
                        }
                    }
                } else {
                    return Err(ApplicationError::UnmatchedTokenInOutput(pattern.clone()));
                }
            },
            Pattern::Selection(Selection { id: Some(id), options, .. }) => {
                if let Some(choice) = choices.selection().get(id) {
                    if let Some(content) = options.get(*choice) {
                        for phone in patterns_to_phones(content.inner(), choices)? {
                            phones.push(phone);
                        }
                    } else {
                        return Err(ApplicationError::InvalidSelectionAccess(pattern.clone(), *choice))
                    }
                } else {
                    return Err(ApplicationError::UnmatchedTokenInOutput(pattern.clone()));
                }
            },
            Pattern::Gap { .. } => return Err(ApplicationError::GapOutOfCond),
            _ => return Err(ApplicationError::UnmatchedTokenInOutput(pattern.clone()))
        }
    }

    Ok(phones)
}

/// Errors that occur when trying to apply a rule
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum ApplicationError<'s> {
    UnmatchedTokenInOutput(Pattern<'s>),
    InvalidSelectionAccess(Pattern<'s>, usize),
    ExceededLimit(LimitCondition),
    GapOutOfCond,
    PatternCannotBeConvertedToPhones(Pattern<'s>),
}

impl std::error::Error for ApplicationError<'_> {}

impl std::fmt::Display for ApplicationError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSelectionAccess(scope, elem) => {
                write!(f, "Cannot access element {} in scope: {scope}", elem + 1)
            },
            Self::UnmatchedTokenInOutput(pattern) => {
                write!(f, "Cannot match the following token in the output to a token in the input: {pattern}\nConsider adding a label '{}' and ensuring it is used in the input or every condition", IrToken::Label("name"))
            },
            Self::ExceededLimit(limit) => write!(f, "{}", match limit {
                LimitCondition::Time(_) => "Could not apply changes in allotted time",
                LimitCondition::Count { attempts: _, max: _ } => "Could not apply changes with the allotted application attempts",
            }),
            Self::GapOutOfCond => write!(f, "{}", RuleStructureError::GapOutOfCond),
            Self::PatternCannotBeConvertedToPhones(pattern) => write!(f, "'{pattern}' cannot be converted to a phone or list of phones"),
        }
    }
}