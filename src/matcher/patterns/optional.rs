use crate::{matcher::{choices::{Choices, OwnedChoices}, match_state::MatchState, patterns::list::PatternList, phones::Phones}, rules::tokens::ScopeId, tokens::ScopeType};

/// A pattern the represents the potential of a sub-pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Optional<'r, 's> {
    pub(super) selected: bool,
    pub(super) option: PatternList<'r, 's>,
    pub(super) id: Option<&'r ScopeId<'s>>,
}

impl<'r, 's: 'r> MatchState<'r, 's> for Optional<'r, 's> {
    fn matches(&self, phones: &mut Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        if let Some(id) = self.id {
            if let Some(choice) = choices.optional.get(id).copied() {
                // if choice and selection do not align, the match fails
                if self.selected != choice {
                    return None;
                }

                // checks the match if the choice aligns with the selection
                if choice {
                    // checks if the option matches
                    self.option.matches(phones, choices)
                } else {
                    // if the option is not inserted the pattern matches
                    Some(OwnedChoices::default())
                }
            } else {
                // chooses the selection and checks it
                let mut new_choices = choices.partial_clone();
                new_choices.optional.to_mut().insert(id, self.selected);

                // checks if the option matches with the new selection
                if self.selected {
                    let internal_choices = self.option.matches(phones, &new_choices)?;
                    new_choices.take_owned(internal_choices);
                }

                Some(new_choices.owned_choices())
            }
        } else if self.selected {
            // checks if the option matches
            self.option.matches(phones, choices)
        } else {
            // if the option is not inserted the pattern matches
            Some(OwnedChoices::default())
        }
    }

    fn next_match(&mut self, phones: &Phones<'_, 's>, choices: &Choices<'_, 'r, 's>) -> Option<OwnedChoices<'r, 's>> {
        if self.selected {
            loop {
                if self.option.next_match(phones, choices).is_some() {
                    if let Some(new_choices) = self.matches(&mut phones.clone(), choices) {
                        return Some(new_choices);
                    }

                    continue;
                }

                break;
            }

            self.selected = false;
            self.option.reset();
        } else {
            return None
        }
            
        self.matches(&mut phones.clone(), choices)
    }

    fn len(&self) -> usize {
        if self.selected {
            self.option.len()
        } else{
            0
        }
    }

    fn reset(&mut self) {
        self.selected = true;
        self.option.reset();
    }
}

impl std::fmt::Display for Optional<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = self.id {
            write!(f, "{id}")?;
        }

        write!(f, "{} {} {}", ScopeType::Optional.fmt_start(), self.option, ScopeType::Optional.fmt_end())
    }
}
