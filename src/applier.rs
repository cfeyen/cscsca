use matcher::{has_empty_form, match_len, tokens_match_phones_from_left, tokens_match_phones_from_right, Choices, MatchError};

use crate::{meta_tokens::{Direction, Shift, ShiftType}, phones::Phone, rules::sound_change_rule::{RuleToken, SoundChangeRule}, tokens::ir::IrToken, BOUND_STR};

mod matcher;

#[cfg(test)]
mod tests;

#[cfg(feature = "async_apply")]
pub mod async_applier;

/// Applies a rule to a list of phones
pub fn apply<'a, 's>(rule: &'a SoundChangeRule<'s>, phones: &mut Vec<Phone<'s>>) -> Result<(), ApplicationError<'a, 's>> {
    let dir = rule.kind.dir;
    let mut phone_index = dir.start_index(phones);
    
    while phone_index < phones.len() {
        if let Some((replace_len, input_len)) = apply_at(rule, phones, phone_index)? {
            phone_index = next_position(rule, input_len, replace_len, phone_index, phones);
        } else {
            phone_index = dir.change_by_one(phone_index);
        }
    }

    Ok(())
}

fn next_position(rule: &SoundChangeRule, input_len: usize, replace_len: usize, phone_index: usize, phones: &[Phone]) -> usize {
    let dir = rule.kind.dir;
    match (dir, rule.kind.kind) {
        (Direction::LTR, ShiftType::Move) => {
            dir.change_by(phone_index, replace_len)
        }
        (Direction::RTL, _) if phone_index >= phones.len() => {
            // ensures removing a phone does not take the phone index out of the phone list ending the rule early
            phones.len().wrapping_sub(1)
        }
        (Direction::RTL, ShiftType::Move) => {
            dir.change_by(phone_index, input_len)
        }
        _ => phone_index
    }
}

/// Applies a rule to a location in a list of phones if the input and conds match
/// 
/// Return: (the length of the output, the length of what it replaced)
fn apply_at<'a, 's: 'a>(rule: &'a SoundChangeRule<'s>, phones: &mut Vec<Phone<'s>>, phone_index: usize) -> Result<Option<(usize, usize)>, ApplicationError<'a, 's>> {
    let SoundChangeRule {
        kind,
        input,
        output,
        conds,
        anti_conds,
    } = rule;

    if input.is_empty() || has_empty_form(input) { Err(MatchError::EmptyInput)?; }

    let mut choices = Choices::default();

    let Shift { dir, kind: _} = kind;

    let matches = if *dir == Direction::LTR {
        tokens_match_phones_from_left(&mut input.iter().collect::<Vec<_>>(),&phones[phone_index..], &mut choices)?
    } else{
        tokens_match_phones_from_right(&mut input.iter().collect::<Vec<_>>(), &phones[0..=phone_index], &mut choices)?
    };

    if !matches { return Ok(None); }

    let input_len = match_len(input, &choices)?;

    'cond_loop: for cond in conds.iter() {
        // saves choices to reset between conditions
        // ? this process could probably be optimized
        let initial_choices = choices.clone();

        if *dir == Direction::LTR {
            let before_matches = tokens_match_phones_from_right(&mut cond.before.iter().collect::<Vec<_>>(), &phones[0..phone_index], &mut choices)?;
            let after_matches = tokens_match_phones_from_left(&mut cond.after.iter().collect::<Vec<_>>(), &phones[phone_index + input_len..], &mut choices)?;

            if before_matches && after_matches {
                for anti_cond in anti_conds.iter() {
                    // saves choices to reset between anti-conditions
                    let initial_choices = choices.clone();

                    let before_matches = tokens_match_phones_from_right(&mut anti_cond.before.iter().collect::<Vec<_>>(), &phones[0..phone_index], &mut choices)?;
                    let after_matches = tokens_match_phones_from_left(&mut anti_cond.after.iter().collect::<Vec<_>>(), &phones[phone_index + input_len..], &mut choices)?;

                    if before_matches && after_matches {
                        continue 'cond_loop;
                    }

                    // resets choices between anti-conditions
                    choices = initial_choices;
                }

                return replace_input(phones, phone_index, input_len, output, &choices);
            }
        } else {
            let before_slice = if input_len <= phone_index {
                &phones[0..=phone_index - input_len]
            } else {
                &[]
            };
            
            let before_matches = tokens_match_phones_from_right(&mut cond.before.iter().collect::<Vec<_>>(), before_slice, &mut choices)?;
            let after_matches = tokens_match_phones_from_left(&mut cond.after.iter().collect::<Vec<_>>(), &phones[phone_index + 1..], &mut choices)?;

            if before_matches && after_matches {
                for anti_cond in anti_conds.iter() {
                    // saves choices to reset between anti-conditions
                    let initial_choices = choices.clone();

                    let before_matches = tokens_match_phones_from_right(&mut anti_cond.before.iter().collect::<Vec<_>>(), before_slice, &mut choices)?;
                    let after_matches = tokens_match_phones_from_left(&mut anti_cond.after.iter().collect::<Vec<_>>(), &phones[phone_index + 1..], &mut choices)?;

                    if before_matches && after_matches {
                        continue 'cond_loop;
                    }

                    // resets choices between anti-conditions
                    choices = initial_choices;
                }

                return replace_input(phones, 1 + phone_index - input_len, input_len, output, &choices);
            }
        }

        // resets choices between conditions
        choices = initial_choices;
    }

    Ok(None)
}

/// Replaces the slice phones[index..input_len] with the output as phones
/// 
/// Return: (the length of the output, the length of what it replaced)
fn replace_input<'a, 's: 'a>(phones: &mut Vec<Phone<'s>>, index: usize, input_len: usize, output: &'a [RuleToken<'s>], choices: &Choices<'a, 's>) -> Result<Option<(usize, usize)>, ApplicationError<'a, 's>> {
    let mut shifted_phones = Vec::new();

    let phone_iter = &mut phones.iter();

    // adds the proceeding phones to the new phones
    for &phone in phone_iter.take(index) {
        shifted_phones.push(phone);
    }

    // discards the input
    phone_iter.take(input_len).for_each(|_| ()); // since take is lazy, the for each causes it to be used

    // number of duplicated bound strs removed
    let mut reductions = 0;

    // adds the output
    for phone in tokens_to_phones(output, choices)? {
        if shifted_phones.last() == Some(&Phone::new(BOUND_STR)) && phone == Phone::new(BOUND_STR) {
            reductions += 1;
        } else {
            shifted_phones.push(phone);
        }
    }

    // adds the following phones
    for &phone in phone_iter {
        shifted_phones.push(phone);
    }

    let mut new_phones = Vec::new();

    // prevents bounds from doubling up
    for phone in shifted_phones {
        if !(new_phones.last() == Some(&Phone::new(BOUND_STR)) && phone == Phone::new(BOUND_STR)) {
            new_phones.push(phone);
        }
    }

    // allows stay motion to move if no change is made, should prevent some infinite loops
    if new_phones == *phones { return Ok(None); }

    *phones = new_phones;

    Ok(Some((match_len(output, choices)? - reductions, input_len)))
}

/// Converts rule tokens to the phones that they represent according to choices that have been made
fn tokens_to_phones<'a, 's: 'a>(tokens: &'a [RuleToken<'s>], choices: &Choices<'a, 's>) -> Result<Vec<Phone<'s>>, ApplicationError<'a, 's>> {
    let mut phones = Vec::new();

    for token in tokens {
        match token {
            RuleToken::Phone(phone) => phones.push(*phone),
            RuleToken::Any { id: Some(id) } => {
                if let Some(phone) = choices.any_choices.get(id) {
                    phones.push(*phone);
                } else {
                    return Err(ApplicationError::UnmatchedTokenInOutput(token));
                }
            },
            RuleToken::OptionalScope { id: Some(id), content } => {
                if let Some(insert) = choices.optional_choices.get(id) {
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
                if let Some(choice) = choices.selection_choices.get(id) {
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
            _ => return Err(ApplicationError::UnmatchedTokenInOutput(token))
        };
    }

    Ok(phones)
}

/// Errors that occur when trying to apply a rule
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum ApplicationError<'a, 's: 'a> {
    MatchError(MatchError<'a, 's>),
    UnmatchedTokenInOutput(&'a RuleToken<'s>),
    InvalidSelectionAccess(&'a RuleToken<'s>, usize),
}

impl<'a, 's> From<MatchError<'a, 's>> for ApplicationError<'a, 's> {
    fn from(value: MatchError<'a, 's>) -> Self {
        Self::MatchError(value)
    }
}

impl std::error::Error for ApplicationError<'_, '_> {}

impl std::fmt::Display for ApplicationError<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::InvalidSelectionAccess(scope, elem) => {
                format!("Cannot access element {} in scope: {scope}", elem + 1)
            },
            Self::MatchError(e) => format!("{e}"),
            Self::UnmatchedTokenInOutput(token) => {
                format!("Cannot match the following token in the output to a token in the input: {token}\nConsider adding a label '{}' and ensuring it is used in the input or every condition", IrToken::Label("name"))
            }
        };

        write!(f, "{}", s)
    }
}