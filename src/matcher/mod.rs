use std::{borrow::Cow, collections::HashMap};

use crate::{
    keywords::MATCH_CHAR,
    ir::tokens::IrToken,
    phones::Phone,
    rules::tokens::{RuleToken, ScopeId},
    tokens::Direction
};

#[cfg(test)]
mod ltr_tests;

#[cfg(test)]
mod rtl_tests;

#[cfg(test)]
mod len_tests;

#[cfg(test)]
mod empty_form_tests;


/// Checks if tokens match phones starting from the left
pub fn tokens_match_phones_from_right<'r, 's>(tokens: &'r [RuleToken<'s>], phones: &[Phone<'s>], choices: &mut Choices<'_, 'r, 's>) -> Result<bool, MatchError<'r, 's>> {
    MatchEnviroment::new(tokens, phones, Direction::Rtl).tokens_match_phones(choices)
}

/// Checks if tokens match phones starting from the left
pub fn tokens_match_phones_from_left<'r, 's>(tokens: &'r [RuleToken<'s>], phones: &[Phone<'s>], choices: &mut Choices<'_, 'r, 's>) -> Result<bool, MatchError<'r, 's>> {
    MatchEnviroment::new(tokens, phones, Direction::Ltr).tokens_match_phones(choices)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchEnviroment<'r, 's, 'p> {
    tokens: &'r [RuleToken<'s>],
    token_index: usize,
    phones: &'p [Phone<'s>],
    phone_index: usize,
    direction: Direction,
}

impl<'r, 's, 'p> MatchEnviroment<'r, 's, 'p> {
    /// Creates a new `MatchEnviroment`
    pub fn new(tokens: &'r [RuleToken<'s>], phones: &'p [Phone<'s>], direction: Direction) -> Self {
        Self {
            tokens,
            token_index: direction.start_index(tokens),
            phones,
            phone_index: direction.start_index(phones),
            direction,
        }
    }

    /// Copies `self` but replaces the `tokens` and `token_index`
    /// with `new_tokens` and its starting index
    fn with_new_tokens(&self, new_tokens: &'r [RuleToken<'s>]) -> Self {
        Self {
            tokens: new_tokens,
            token_index: self.direction.start_index(new_tokens),
            phones: self.phones,
            phone_index: self.phone_index,
            direction: self.direction,
        }
    }

    /// Gets the phone at the phone index
    fn get_phone(&self) -> Option<&'p Phone<'s>> {
        self.phones.get(self.phone_index)
    }

    /// Increments the token index by one in the direction of the match
    fn inc_token_index(&mut self) {
        self.token_index = self.direction.change_by_one(self.token_index);
    }

    /// Increments the phone index by one in the direction of the match
    fn inc_phone_index(&mut self) {
        self.phone_index = self.direction.change_by_one(self.phone_index);
    }

    /// Checks if the token enviroment matches the phones
    /// 
    /// ## Side Effects
    /// - may mutate the `token_index` and/or `phone_index` fields
    /// - may mutate `choices`
    pub fn tokens_match_phones<'c>(&mut self, choices: &mut Choices<'c, 'r, 's>) -> Result<bool, MatchError<'r, 's>> {
        let Some(token) = self.tokens.get(self.token_index) else {
            return Ok(true);
        };

        match token {
            RuleToken::Phone(phone) => {
                if phone.matches(self.get_phone().unwrap_or(&Phone::Bound)) {
                    self.inc_phone_index();
                    self.inc_token_index();
                    self.tokens_match_phones(choices)
                } else {
                    Ok(false)
                }
            }
            RuleToken::Any { id } => {
                if Self::any_matches_phone(id.as_ref(), self.get_phone(), choices) {
                    self.inc_phone_index();
                    self.inc_token_index();
                    self.tokens_match_phones(choices)
                } else {
                    Ok(false)
                }
            },
            RuleToken::Gap { id }
                => self.gap_and_after_match_phones(id.as_ref().copied(), choices),
            RuleToken::OptionalScope { id, content }
                => self.optional_and_after_match_phones(id.as_ref(), content, choices),
            RuleToken::SelectionScope { id, options }
                => self.selection_and_after_match_phones(id.as_ref(), options, choices),
        }
    }

    /// Checks if a selection scope and all following tokens match a list of phones
    /// based on the scope's id and options
    fn selection_and_after_match_phones<'c>(&mut self, id: Option<&'r ScopeId<'s>>, options: &'r [Vec<RuleToken<'s>>], choices: &mut Choices<'c, 'r, 's>) -> Result<bool, MatchError<'r, 's>> {
        if let Some(id) = id
            && let Some(choice) = choices.selection.get(id).copied()
        {
            let Some(content) = options.get(choice) else {
                return Err(MatchError::InvalidSelectionChoice(id.clone(), choice));
            };

            let mut content_env = self.with_new_tokens(content);

            let choice_matches = content_env.tokens_match_phones(choices)?;
            self.phone_index = content_env.phone_index;
            self.inc_token_index();

            return Ok(choice_matches && self.tokens_match_phones(choices)?);
        }

        let starting_phone_index = self.phone_index;

        for (option_num, option) in options.iter().enumerate() {
            self.phone_index = starting_phone_index;

            let mut option_env = self.with_new_tokens(option);

            let mut new_choices = choices.partial_clone();
            if let Some(id) = id {
                new_choices.selection.to_mut().insert(id, option_num);
            }

            if option_env.tokens_match_phones(&mut new_choices)? {
                let starting_token_index = self.token_index;
                self.inc_token_index();
                self.phone_index = option_env.phone_index;

                if self.tokens_match_phones(&mut new_choices)? {
                    let owned_new_choices = new_choices.owned_choices();
                    choices.take_owned(owned_new_choices);

                    return Ok(true);
                }

                self.token_index = starting_token_index;
            }
        }

        Ok(false)
    }

    /// Checks if a gap and all following tokens match a list of phones
    /// based on the gap's id
    fn gap_and_after_match_phones<'c>(&mut self, id: Option<&'s str>, choices: &mut Choices<'c, 'r, 's>) -> Result<bool, MatchError<'r, 's>> {
        for len in 0.. {
            if len > 0 {
                let pre_phone_index = self.direction.change_by(self.phone_index, len - 1);
                if self.phones.get(pre_phone_index).unwrap_or(&Phone::Bound) == &Phone::Bound {
                    break;
                }
            }

            let mut new_choices = choices.partial_clone();

            if let Some(id) = id {
                if let Some(max_len) = choices.gap.get(id).copied() && len > max_len {
                    break;
                }

                new_choices.gap.to_mut().insert(id, len);
            }

            let mut after_env = *self;
            after_env.inc_token_index();
            after_env.phone_index = after_env.direction.change_by(after_env.phone_index, len);

            if after_env.tokens_match_phones(&mut new_choices)? {
                let owned_new_choices = new_choices.owned_choices();
                choices.take_owned(owned_new_choices);

                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Checks if an optional scope and all following tokens match a list of phones
    /// based on the scope's id and contents
    fn optional_and_after_match_phones<'c>(&mut self, id: Option<&'r ScopeId<'s>>, content: &'r [RuleToken<'s>], choices: &mut Choices<'c, 'r, 's>) -> Result<bool, MatchError<'r, 's>> {
        let starting_phone_index = self.phone_index;
        let starting_token_index = self.token_index;

        let mut after_env = *self;
        after_env.inc_token_index();

        let mut content_env = self.with_new_tokens(content);

        if let Some(id) = id {
            if let Some(choice) = choices.optional.get(id).copied() {
                if choice {
                    let content_matches = content_env.tokens_match_phones(choices)?;
                    if !content_matches {
                        return Ok(false);
                    }

                    after_env.phone_index = content_env.phone_index;
                }

                return after_env.tokens_match_phones(choices);
            }
        }

        let mut new_choices = choices.partial_clone();

        let content_matches = content_env.tokens_match_phones(&mut new_choices)?;
        
        if content_matches {
            after_env.phone_index = content_env.phone_index;
            if let Some(id) = id {
                new_choices.optional.to_mut().insert(id, true);
            }

            if after_env.tokens_match_phones(&mut new_choices)? {
                let owned_new_choices = new_choices.owned_choices();
                choices.take_owned(owned_new_choices);
                
                return Ok(true);
            }
        }
        
        self.phone_index = starting_phone_index;
        self.token_index = starting_token_index;
        self.inc_token_index();

        if let Some(id) = id {
            choices.optional.to_mut().insert(id, false);
        }

        self.tokens_match_phones(choices)
    }

    /// Checks if an any matches a phone based on its id
    fn any_matches_phone<'c>(id: Option<&'r ScopeId<'s>>, phone: Option<&Phone<'s>>, choices: &mut Choices<'c, 'r, 's>) -> bool {
        let Some(phone) = phone.copied() else {
            return false;
        };

        if phone == Phone::Bound {
            return false;
        }

        if let Some(id) = id {
            if let Some(choice) = choices.any.get(id).copied() {
                phone == choice
            } else {
                choices.any.to_mut().insert(id, phone);
                true
            }
        } else {
            true
        }
    }
}

/// Returns the number of phones the tokens match to using the choices as reference
/// 
/// Note: Should only be used on inputs and outputs, not conditions
pub fn match_len<'r, 's>(tokens: &'r [RuleToken<'s>], choices: &Choices<'_, 'r, 's>) -> Result<usize, MatchError<'r, 's>> {
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
                        return Err(MatchError::InvalidSelectionChoice(id.clone(), *choice));
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

/// Choices for how agreement should occur
#[derive(Debug, Clone, Default)]
pub struct Choices<'c, 'r, 's> {
    selection: Cow<'c, HashMap<&'r ScopeId<'s>, usize>>,
    optional: Cow<'c, HashMap<&'r ScopeId<'s>, bool>>,
    any: Cow<'c, HashMap<&'r ScopeId<'s>, Phone<'s>>>,
    gap: Cow<'c, HashMap<&'s str, usize>>,
}

impl<'c, 'r, 's> Choices<'c, 'r, 's> {
    /// Gets the selection scope choices
    pub fn selection(&self) -> &HashMap<&'r ScopeId<'s>, usize> {
        &self.selection
    }

    /// Gets the optional scope choices
    pub fn optional(&self) -> &HashMap<&'r ScopeId<'s>, bool> {
        &self.optional
    }

    /// Gets the any phone choices
    pub fn any(&self) -> &HashMap<&'r ScopeId<'s>, Phone<'s>> {
        &self.any
    }

    /// A cheeper way to clone `Choices` with less heap allocation
    pub fn partial_clone(&'c self) -> Self {
        Self {
            selection: Cow::Borrowed(&*self.selection),
            optional: Cow::Borrowed(&*self.optional),
            any: Cow::Borrowed(&*self.any),
            gap: Cow::Borrowed(&*self.gap),
        }
    }

    /// Converts a set of copy-on-write choices to only the owned choices
    fn owned_choices(self) -> OwnedChoices<'r, 's> {
        OwnedChoices {
            selection: take_owned_from_cow(self.selection),
            optional: take_owned_from_cow(self.optional),
            any: take_owned_from_cow(self.any),
            gap: take_owned_from_cow(self.gap),
        }
    }

    /// Takes the choices from `owned`
    fn take_owned(&mut self, owned: OwnedChoices<'r, 's>) {
        if let Some(selection) = owned.selection {
            self.selection = Cow::Owned(selection);
        }

        if let Some(optional) = owned.optional {
            self.optional = Cow::Owned(optional);
        }

        if let Some(any) = owned.any {
            self.any = Cow::Owned(any);
        }

        if let Some(gap) = owned.gap {
            self.gap = Cow::Owned(gap);
        }
    }
}

/// A varient of `Choices` where each map is either owned or does not exist
///
/// Used to optimise some clones
#[derive(Debug)]
struct OwnedChoices<'r, 's> {
    selection: Option<HashMap<&'r ScopeId<'s>, usize>>,
    optional: Option<HashMap<&'r ScopeId<'s>, bool>>,
    any: Option<HashMap<&'r ScopeId<'s>, Phone<'s>>>,
    gap: Option<HashMap<&'s str, usize>>,
}

/// Returns the owned content of a `Cow` if it exists
fn take_owned_from_cow<T: Clone>(cow: Cow<'_, T>) -> Option<T> {
    if let Cow::Owned(t) = cow {
        Some(t)
    } else {
        None
    }
}

/// Errors that occur when trying to match tokens to phones
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum MatchError<'r, 's> {
    EmptyInput,
    InvalidSelectionChoice(ScopeId<'s>, usize),
    UnlabeledScope(&'r RuleToken<'s>),
    CannotCheckLenOfGap,
    LeftMustBePhones(&'r RuleToken<'s>),
}

impl std::error::Error for MatchError<'_, '_> {}

impl std::fmt::Display for MatchError<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::EmptyInput => "Input does not always contain phones".to_string(),
            Self::InvalidSelectionChoice(ScopeId::Name(id), num) => {
                format!("Tried to access option {} of a selection {id} in the output where none exists", num + 1)
            },
            Self::InvalidSelectionChoice(_, num) => {
                // node: this isn't checked here but is should still be checked when applying
                format!("Tried to access option {} of a selection in the output where none exists", num + 1)
            },
            Self::UnlabeledScope(scope) => {
                format!("Cannot resove scope as a value\nTry adding a label '{}' before the scope and ensuring it is used in the input\nScope:\t{scope}", IrToken::Label("name"))
            },
            Self::CannotCheckLenOfGap => format!("Cannot check the length of '{}' in an input", RuleToken::Gap { id: None }),
            Self::LeftMustBePhones(token) => format!("The left side of '{MATCH_CHAR}' may only contain phones, found: {token}")
        };

        write!(f, "{s}")
    }
}