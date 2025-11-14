use std::{borrow::Cow, collections::HashMap};

use crate::{phones::Phone, tokens::ScopeId};

/// Choices for how agreement should occur
#[derive(Debug, Clone, Default)]
pub struct Choices<'c, 's> {
    pub(super) selection: Cow<'c, HashMap<ScopeId<'s>, usize>>,
    pub(super) optional: Cow<'c, HashMap<ScopeId<'s>, bool>>,
    pub(super) repetition: Cow<'c, HashMap<&'s str, usize>>,
    pub(super) any: Cow<'c, HashMap<ScopeId<'s>, Phone<'s>>>,
}

impl<'c, 's> Choices<'c, 's> {
    /// Gets the selection scope choices
    pub fn selection(&self) -> &HashMap<ScopeId<'s>, usize> {
        &self.selection
    }

    /// Gets the optional scope choices
    pub fn optional(&self) -> &HashMap<ScopeId<'s>, bool> {
        &self.optional
    }

    /// Gets the repetition choices
    pub fn repetition(&self) -> &HashMap<&'s str, usize> {
        &self.repetition
    }

    /// Gets the any phone choices
    pub fn any(&self) -> &HashMap<ScopeId<'s>, Phone<'s>> {
        &self.any
    }

    /// A cheeper way to clone `Choices` with less heap allocation
    pub fn partial_clone(&'c self) -> Self {
        Self {
            selection: Cow::Borrowed(&*self.selection),
            optional: Cow::Borrowed(&*self.optional),
            repetition: Cow::Borrowed(&*self.repetition),
            any: Cow::Borrowed(&*self.any),
        }
    }

    /// Converts a set of copy-on-write choices to only the owned choices
    pub fn owned_choices(self) -> OwnedChoices<'s> {
        OwnedChoices {
            selection: take_owned_from_cow(self.selection),
            optional: take_owned_from_cow(self.optional),
            repetition: take_owned_from_cow(self.repetition),
            any: take_owned_from_cow(self.any),
        }
    }

    /// Takes the choices from `owned`
    pub fn take_owned(&mut self, owned: OwnedChoices<'s>) {
        if let Some(selection) = owned.selection {
            self.selection = Cow::Owned(selection);
        }

        if let Some(optional) = owned.optional {
            self.optional = Cow::Owned(optional);
        }

        if let Some(repetition) = owned.repetition {
            self.repetition = Cow::Owned(repetition);
        }

        if let Some(any) = owned.any {
            self.any = Cow::Owned(any);
        }
    }
}

/// A varient of `Choices` where each map is either owned or does not exist
///
/// Used to optimise some clones
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OwnedChoices<'s> {
    selection: Option<HashMap<ScopeId<'s>, usize>>,
    optional: Option<HashMap<ScopeId<'s>, bool>>,
    repetition: Option<HashMap<&'s str, usize>>,
    any: Option<HashMap<ScopeId<'s>, Phone<'s>>>,
}

/// Returns the owned content of a `Cow` if it exists
fn take_owned_from_cow<T: Clone>(cow: Cow<'_, T>) -> Option<T> {
    if let Cow::Owned(t) = cow {
        Some(t)
    } else {
        None
    }
}