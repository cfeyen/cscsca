use std::collections::HashMap;

use crate::{meta_tokens::Direction, phones::Phone, rules::sound_change_rule::{RuleToken, ScopeId}, tokens::ir::IrToken};

#[cfg(test)]
mod ltr_tests;

#[cfg(test)]
mod rtl_tests;

#[cfg(test)]
mod len_tests;

#[cfg(test)]
mod empty_form_tests;

/// Checks if tokens match phones starting from the left
/// 
/// Note see `tokens_match_phones` for side effects
pub fn tokens_match_phones_from_right<'a, 's: 'a>(tokens: &'a [RuleToken<'s>], phones: &[Phone<'s>], choices: &mut Choices<'a, 's>) -> Result<bool, MatchError<'a, 's>> {
    tokens_match_phones(tokens, phones, Direction::Rtl.start_index(tokens), &mut Direction::Rtl.start_index(phones), choices, Direction::Rtl)
}

/// Checks if tokens match phones starting from the left
/// 
/// Note see `tokens_match_phones` for side effects
pub fn tokens_match_phones_from_left<'a, 's: 'a>(tokens: &'a [RuleToken<'s>], phones: &[Phone<'s>], choices: &mut Choices<'a, 's>) -> Result<bool, MatchError<'a, 's>> {
    tokens_match_phones(tokens, phones, 0, &mut Direction::Ltr.start_index(phones), choices, Direction::Ltr)
}

/// Checks if tokens match phones moving according to the direction starting at the provided token and phone indexes
/// 
/// ## Side Effects
/// - if there are the tokens match, the provided hash maps contain agreements
/// - if there are errors there may be bad data in the maps
fn tokens_match_phones<'a, 's: 'a>(tokens: &'a [RuleToken<'s>], phones: &[Phone<'s>], token_index: usize, phone_index: &mut usize, choices: &mut Choices<'a, 's>, direction: Direction) -> Result<bool, MatchError<'a, 's>> {
    let Some(token) = tokens.get(token_index) else {
        return Ok(true);
    };

    match token {
        // if the phone matches, check the next token with the next index, else return false
        RuleToken::Phone(phone) => if phone.matches(phones.get(*phone_index).unwrap_or(&Phone::Bound)) {
            *phone_index = direction.change_by_one(*phone_index);
            tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)
        } else {
            Ok(false)
        },
        RuleToken::SelectionScope {
            id: Some(id), options
        // if the id is alredy defined, try to access the index
        } if choices.selection.contains_key(id) => {
            let choice = choices.selection[id];
            // if the index can be accessed use the contents at that index
            if let Some(content) = options.get(choice) {
                // if those contents can be matched, continue checking, otherwise return false
                let start_index = direction.start_index(content);
                if tokens_match_phones(content, phones, start_index, phone_index, choices, direction)? {
                    tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)
                } else {
                    Ok(false)
                }
            } else { // if the index cannot be accessed return an error
                let name = if let ScopeId::Name(name) = *id {
                    Some(name)
                } else {
                    None
                };

                Err(MatchError::InvalidSelectionChoice(name, choice))
            }
        },
        RuleToken::SelectionScope {
            id: Some(id), options
        // if the id is not defined, try to define it
        } => {
            for (num, option) in options.iter().enumerate() {
                choices.selection.insert(id, num);
                let starting_phone_index = *phone_index;

                let start_index = direction.start_index(option);

                let option_matches = tokens_match_phones(option, phones, start_index, phone_index, choices, direction)?;
                let following_matches = tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)?;

                if option_matches && following_matches {
                    return Ok(true);
                }

                *phone_index = starting_phone_index;
                choices.selection.remove(id);
            }

            Ok(false)
        },
        RuleToken::SelectionScope {
            id: None, options
        } => {
            for option in options {
                let starting_phone_index = *phone_index;

                let start_index = direction.start_index(option);

                let option_matches = tokens_match_phones(option, phones, start_index, phone_index, choices, direction)?;
                let following_matches = tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)?;

                if option_matches && following_matches {
                    return Ok(true);
                }

                *phone_index = starting_phone_index;
            }

            Ok(false)
        }
        RuleToken::Any { id: None } => {
            let bound_phone = Phone::Bound;
            let phone = phones.get(*phone_index).unwrap_or(&bound_phone);

            if phone == &Phone::Bound {
                Ok(false)
            } else {
                *phone_index = direction.change_by_one(*phone_index);
                tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)
            }
        },
        RuleToken::Any { id: Some(id) } => {
            let bound_phone = Phone::Bound;
            let phone = phones.get(*phone_index).unwrap_or(&bound_phone);

            if let Some(choice) = choices.any.get(id) {
                if choice == phone {
                    *phone_index = direction.change_by_one(*phone_index);
                    tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)
                } else {
                    Ok(false)
                }
            } else if phone != &Phone::Bound {
                *phone_index = direction.change_by_one(*phone_index);
                choices.any.insert(id, *phone);
                tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)
            } else {
                Ok(false)
            }
        },
        RuleToken::Gap { id: None } => {
            while let Some(phone) = phones.get(*phone_index) {
                let starting_index = *phone_index;

                if *phone == Phone::Bound {
                    break;
                } else if tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)? {
                    return Ok(true);
                }

                *phone_index = direction.change_by_one(starting_index);
            }

            Ok(false)
        },
        RuleToken::Gap { id: Some(id) } => {
            let target_len = choices.gap.get(id).copied();
            let mut len = 0;

            while let Some(phone) = phones.get(*phone_index) {
                let starting_index = *phone_index;

                if target_len.is_none() {
                    choices.gap.insert(id, len);
                }

                if target_len.map(|t| len > t) == Some(true) || phone == &Phone::Bound {
                    break;
                } else if tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)? {
                    return Ok(true);
                }

                *phone_index = direction.change_by_one(starting_index);
                len += 1;
            }

            Ok(false)
        },
        RuleToken::OptionalScope {
            id: Some(id), content
        } if choices.optional.contains_key(id) => {
            let choice = choices.optional[id];

            let start_index = direction.start_index(content);
            if choice {
                if tokens_match_phones(content, phones, start_index, phone_index, choices, direction)? {
                    tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)
                } else {
                    Ok(false)
                }
            } else {
                tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)
            }
        },
        RuleToken::OptionalScope {
            id: Some(id), content
        } => {
            let starting_phone_index = *phone_index;
            let start_index = direction.start_index(content);

            choices.optional.insert(id, true);

            let content_matches = tokens_match_phones(content, phones, start_index, phone_index, choices, direction)?;
            let following_matches = tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)?;

            if content_matches && following_matches {
                Ok(true)
            } else {
                *phone_index = starting_phone_index;
                choices.optional.insert(id, false);
                tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)
            }
        },
        RuleToken::OptionalScope {
            id: None, content
        } => {
            let starting_phone_index = *phone_index;
            let start_index = direction.start_index(content);

            let content_matches = tokens_match_phones(content, phones, start_index, phone_index, choices, direction)?;
            let following_matches = tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)?;

            if content_matches && following_matches {
                Ok(true)
            } else {
                *phone_index = starting_phone_index;
                tokens_match_phones(tokens, phones, direction.change_by_one(token_index), phone_index, choices, direction)
            }
        },
    }
}

/// Returns the number of phones the tokens match to using the choices as reference
/// 
/// Note: Should only be used on inputs and outputs, not conditions
pub fn match_len<'a, 's: 'a>(tokens: &'a [RuleToken<'s>], choices: &Choices<'a, 's>) -> Result<usize, MatchError<'a, 's>> {
    let mut len = 0;

    for token in tokens {
        match token {
            RuleToken::Phone(_) | RuleToken::Any { id: _ } => len += 1,
            RuleToken::Gap { id: _ } => return Err(MatchError::CannotCheckLenOfGap),
            RuleToken::OptionalScope { id: Some(id), content } => {
                if let Some(choice) = choices.optional.get(id) {
                    if *choice {
                        len += match_len(content, choices)?;
                    }
                } else {
                    return Err(MatchError::UnlabeledScope(token));
                }
            },
            RuleToken::SelectionScope { id: Some(id), options } => {
                if let Some(choice) = choices.selection.get(id) {
                    if let Some(content) = options.get(*choice) {
                        len += match_len(content, choices)?;
                    } else {
                        let name = if let ScopeId::Name(name) = id {
                            Some(*name)
                        } else {
                            None
                        };

                        return Err(MatchError::InvalidSelectionChoice(name, *choice));
                    }
                } else {
                    return Err(MatchError::UnlabeledScope(token));
                }
            },
            _ => return Err(MatchError::UnlabeledScope(token))
        }
    }

    Ok(len)
}

pub fn has_empty_form(tokens: &[RuleToken]) -> bool {
    'outer: for token in tokens {
        match token {
            RuleToken::OptionalScope { id: _, content: _ } => (),
            RuleToken::SelectionScope { id: _, options } => {
                for option in options {
                    if has_empty_form(option) {
                        continue 'outer;
                    }
                }

                return false;
            },
            _ => return false,
        }
    }

    true
}

#[derive(Debug, Clone, Default)]
pub struct Choices<'a, 's: 'a> {
    pub selection: HashMap<&'a ScopeId<'s>, usize>,
    pub optional: HashMap<&'a ScopeId<'s>, bool>,
    pub any: HashMap<&'a ScopeId<'s>, Phone<'s>>,
    pub gap: HashMap<&'s str, usize>,
}

/// Errors that occur when trying to match tokens to phones
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum MatchError<'a, 's: 'a> {
    EmptyInput,
    InvalidSelectionChoice(Option<&'s str>, usize),
    UnlabeledScope(&'a RuleToken<'s>),
    CannotCheckLenOfGap,
}

impl std::error::Error for MatchError<'_, '_> {}

impl std::fmt::Display for MatchError<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::EmptyInput => "Input does not always contain phones".to_string(),
            Self::InvalidSelectionChoice(None, num) => {
                // node: this isn't checked here but is should still be checked when applying
                format!("Tried to access option {} of a selection in the output where none exists", num + 1)
            },
            Self::InvalidSelectionChoice(Some(id), num) => {
                format!("Tried to access option {} of a selection {id} in the output where none exists", num + 1)
            },
            Self::UnlabeledScope(scope) => {
                format!("Cannot resove scope as a value\nTry adding a label '{}' before the scope and ensuring it is used in the input\nScope:\t{scope}", IrToken::Label("name"))
            },
            Self::CannotCheckLenOfGap => format!("Cannot check the length of '{}'", RuleToken::Gap { id: None })
        };

        write!(f, "{s}")
    }
}