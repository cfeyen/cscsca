use crate::{
    keywords::ARG_SEP_CHAR,
    matcher::{choices::{Choices, OwnedChoices},
    match_state::MatchState,
    patterns::list::PatternList, phones::Phones},
    tokens::ScopeId,
    tokens::ScopeType,
};

/// A pattern that repersents one of its sub-patterns
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection<'s> {
    /// should always contain at least one item
    pub options: Vec<PatternList<'s>>,
    pub(super) selected_index: usize,
    pub id: Option<ScopeId<'s>>,
}

impl<'s> MatchState<'s>  for Selection<'s> {
    fn matches<'p>(&self, phones: &mut Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        let mut new_choices = choices.partial_clone();

        if let Some(id) = &self.id {
            if let Some(choice) = choices.selection.get(id).copied() {
                if self.selected_index != choice {
                    // selections cannot be changed
                    return None;
                }

                // checks if the choice matches
                let option = self.options.get(choice)?;
                let internal_choices = option.matches(phones, &new_choices)?;

                new_choices.take_owned(internal_choices);
            } else {
                // chooses the current index
                let option = self.options.get(self.selected_index)?;

                new_choices.selection.to_mut().insert(id.clone(), self.selected_index);

                let internal_choices = option.matches(phones, &new_choices)?;

                new_choices.take_owned(internal_choices);
            }
        } else {
            // checks if the selection matches
            let option = self.options.get(self.selected_index)?;
            let internal_choices = option.matches(phones, &new_choices)?;

            new_choices.take_owned(internal_choices);
        }

        Some(new_choices.owned_choices())
    }

    fn next_match<'p>(&mut self, phones: &Phones<'_, 'p>, choices: &Choices<'_, 'p>) -> Option<OwnedChoices<'p>> where 's: 'p {
        loop {
            // checks if the option has a next match form
            if self.options.get_mut(self.selected_index)?.next_match(phones, choices).is_some() {
                // checks if the pattern matches
                if let Some(new_choices) = self.matches(&mut phones.clone(), choices) {
                    return Some(new_choices);
                }
                
                continue;
            }
            
            // if there is not another match, moves to the next option
            self.selected_index += 1;

            // if the next match is invalid, the match fails
            if self.selected_index >= self.options.len() {
                return None;
            }
        }
    }

    fn len(&self) -> usize {
        self.options.get(self.selected_index).map(MatchState::len).unwrap_or_default()
    }

    fn reset(&mut self) {
        self.selected_index = 0;
        self.options.iter_mut().for_each(MatchState::reset);
    }
}

impl std::fmt::Display for Selection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = &self.id {
            write!(f, "{id}")?;
        }

        write!(f, "{} ", ScopeType::Selection.fmt_start())?;

        if let Some(first_option) = self.options.first() {
            write!(f, "{first_option}")?;
        }

        if let Some(rest) = self.options.get(1..) {
            for option in rest {
                write!(f, "{ARG_SEP_CHAR} {option}")?;
            }
        }

        write!(f, " {}", ScopeType::Selection.fmt_end())
    }
}