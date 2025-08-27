use std::time::Instant;

use crate::{
    executor::runtime::LineApplicationLimit,
    ir::tokens::IrToken,
    keywords::GAP_STR,
    matcher::{choices::Choices, pattern::Pattern, rule_pattern::RulePattern, Phones},
    phones::Phone,
    rules::{sound_change_rule::SoundChangeRule, tokens::RuleToken},
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
pub fn apply<'r, 's>(rule: &'r SoundChangeRule<'s>, phones: &mut Vec<Phone<'s>>, limit: Option<LineApplicationLimit>) -> Result<(), ApplicationError<'r, 's>> {
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
fn apply_at<'r, 's>(rule: &'r SoundChangeRule<'s>, phones: &mut Vec<Phone<'s>>, phone_index: usize) -> Result<Option<(usize, usize)>, ApplicationError<'r, 's>> {
    let SoundChangeRule {
        kind,
        input,
        output,
        conds,
        anti_conds,
    } = rule;

    let mut rule_pattern = RulePattern::new(input, conds, anti_conds)?;

    let match_phones = Phones::new(phones, phone_index, kind.dir);

    let mut choices = Choices::default();

    if let Some(new_choices) = rule_pattern.next_match(&match_phones)? {
        choices.take_owned(new_choices);
    } else {
        return Ok(None);
    }

    let input_len = rule_pattern.len();

    replace_input(phones, phone_index, input_len, output, &choices, kind.dir)
}

/// Replaces the slice `phones[index..input_len]` with the output as phones
/// 
/// Return: (the length of the output, the length of what it replaced)
fn replace_input<'r, 's>(phones: &mut Vec<Phone<'s>>, index: usize, input_len: usize, output: &'r [RuleToken<'s>], choices: &Choices<'_, 'r, 's>, dir: Direction) -> Result<Option<(usize, usize)>, ApplicationError<'r, 's>> {
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
    phone_iter.take(input_len).for_each(|_| ()); // since take is lazy, the for each causes it to be used

    let mut output_phones = Vec::new();

    // adds the output
    for phone in tokens_to_phones(output, choices)? {
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

/// Converts rule tokens to the phones that they represent according to choices that have been made
fn tokens_to_phones<'r, 's>(tokens: &'r [RuleToken<'s>], choices: &Choices<'_, 'r, 's>) -> Result<Vec<Phone<'s>>, ApplicationError<'r, 's>> {
    let mut phones = Vec::new();

    for token in tokens {
        match token {
            RuleToken::Phone(phone) => phones.push(*phone),
            RuleToken::Any { id: Some(id) } => {
                if let Some(phone) = choices.any().get(id) {
                    phones.push(*phone);
                } else {
                    return Err(ApplicationError::UnmatchedTokenInOutput(token));
                }
            },
            RuleToken::OptionalScope { id: Some(id), content } => {
                if let Some(insert) = choices.optional().get(id) {
                    if *insert {
                        for phone in tokens_to_phones(content, choices)? {
                            phones.push(phone);
                        }
                    }
                } else {
                    return Err(ApplicationError::UnmatchedTokenInOutput(token));
                }
            },
            RuleToken::SelectionScope { id: Some(id), options } => {
                if let Some(choice) = choices.selection().get(id) {
                    if let Some(content) = options.get(*choice) {
                        for phone in tokens_to_phones(content, choices)? {
                            phones.push(phone);
                        }
                    } else {
                        return Err(ApplicationError::InvalidSelectionAccess(token, *choice))
                    }
                } else {
                    return Err(ApplicationError::UnmatchedTokenInOutput(token));
                }
            },
            RuleToken::Gap { .. } => return Err(ApplicationError::GapOutOfCond),
            _ => return Err(ApplicationError::UnmatchedTokenInOutput(token))
        }
    }

    Ok(phones)
}

/// Errors that occur when trying to apply a rule
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum ApplicationError<'r, 's> {
    UnmatchedTokenInOutput(&'r RuleToken<'s>),
    InvalidSelectionAccess(&'r RuleToken<'s>, usize),
    ExceededLimit(LimitCondition),
    GapOutOfCond,
    PatternCannotBeConvertedToPhones(Pattern<'r, 's>),
}

impl std::error::Error for ApplicationError<'_, '_> {}

impl std::fmt::Display for ApplicationError<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::InvalidSelectionAccess(scope, elem) => {
                format!("Cannot access element {} in scope: {scope}", elem + 1)
            },
            Self::UnmatchedTokenInOutput(token) => {
                format!("Cannot match the following token in the output to a token in the input: {token}\nConsider adding a label '{}' and ensuring it is used in the input or every condition", IrToken::Label("name"))
            },
            Self::ExceededLimit(limit) => match limit {
                LimitCondition::Time(_) => "Could not apply changes in allotted time",
                LimitCondition::Count { attempts: _, max: _ } => "Could not apply changes with the allotted application attempts",
            }.to_string(),
            Self::GapOutOfCond => format!("Gaps ('{GAP_STR}') are not allowed outside of conditions and anti-conditions"),
            Self::PatternCannotBeConvertedToPhones(pattern) => format!("'{pattern}' cannot be converted to a phone or list of phones"),
        };

        write!(f, "{s}")
    }
}