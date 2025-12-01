use std::{cell::RefCell, num::NonZero, rc::Rc};

use crate::{
    ONE, executor::io_events::{IoEvent, RuntimeIoEvent}, ir::{IrLine, tokens::{Break, IrToken}}, matcher::patterns::{
        Pattern, cond::CondPattern, list::PatternList, rule::{RulePattern, SoundChangeRule}
    }, tokens::{AndType, CondType, LabelType, ScopeId, ScopeType, Shift}
};

#[cfg(test)]
mod tests;

/// A rule, executed command, or nothing representing a line of source code
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum RuleLine<'s> {
    Rule {
        rule: SoundChangeRule<'s>,
        lines: NonZero<usize>,
    },
    IoEvent(RuntimeIoEvent<'s>),
    Empty { lines: NonZero<usize> },
}

impl RuleLine<'_> {
    pub const fn lines(&self) -> NonZero<usize> {
        match self {
            Self::Empty { lines } => *lines,
            Self::Rule { lines, .. } => *lines,
            Self::IoEvent(_) => ONE,
        }
    }
}

/// Default ids for unlabled scopes
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct DefaultScopeIds {
    optional: usize,
    selection: usize,
    any: usize,
}

/// Builds a sound change rule out of a line of ir tokens
/// 
/// # Warning:
/// Built time commands should be handled before this function is called
pub fn build_rule(line: IrLine) -> Result<RuleLine, RuleStructureError> {
    let line_count = line.lines();

    let line = match line {
        IrLine::Empty { lines } => return Ok(RuleLine::Empty { lines }),
        IrLine::IoEvent(IoEvent::Tokenizer(_)) => return Ok(RuleLine::Empty { lines: ONE }),
        IrLine::IoEvent(IoEvent::Runtime(cmd)) => return Ok(RuleLine::IoEvent(cmd)),
        IrLine::Ir { tokens, .. } if tokens.is_empty() => return Ok(RuleLine::Empty { lines: ONE }),
        IrLine::Ir { tokens, .. } => tokens
    };

    let (input_region, other_regions) = regionize_ir(&line);
    let mut other_regions = other_regions.into_iter();

    let input = ir_to_input_output(&input_region)?;

    let (shift, output) = match other_regions.next() {
        Some((Break::Shift(shift), output)) =>  (shift, ir_to_input_output(&output)?),
        Some((r#break, _)) => return Err(RuleStructureError::BreakWithoutShift(r#break)),
        None => return Err(RuleStructureError::NoShift),
    };

    let mut conds = Vec::new();
    let mut anti_conds = Vec::new();
    let mut to_anti_conds = false;

    for (r#break, tokens) in other_regions {
        match r#break {
            Break::Shift(shift) => return Err(RuleStructureError::SecondShift(shift)),
            Break::Cond => conds.push(ir_to_cond(&tokens)?),
            Break::AntiCond => {
                to_anti_conds = true;
                anti_conds.push(ir_to_cond(&tokens)?);
            },
            Break::And(and_type) => {
                let cond = ir_to_cond(&tokens)?;

                let last_cond = if to_anti_conds {
                    &mut anti_conds
                } else {
                    &mut conds
                }
                .last_mut()
                .ok_or(RuleStructureError::AndDoesNotFollowCond(and_type))?;

                last_cond.add_and(and_type, cond);
            },
        }
    }

    Ok(RuleLine::Rule {
        rule: SoundChangeRule {
            kind: shift,
            output,
            pattern: RefCell::new(RulePattern::new(PatternList::new(input), conds, anti_conds)?),
        },
        lines: line_count,
    })
}

/// Converts the ir tokens for the input and output of a rule to patterns
#[inline]
fn ir_to_input_output<'s>(ir: &[&IrToken<'s>]) -> Result<Vec<Pattern<'s>>, RuleStructureError<'s>> {
    ir_tokens_to_patterns(
        &mut ir.iter().copied(), 
        Some(&RefCell::default()),
        None, 
        None
    )
}

/// Converts lists of ir tokens for the (anti-)conditions of a rule to a list of `CondPattern`s
fn ir_to_cond<'s>(ir: &[&IrToken<'s>]) -> Result<CondPattern<'s>, RuleStructureError<'s>> {
        let focus = if ir.contains(&&IrToken::CondType(CondType::Pattern)) {
            CondType::Pattern
        } else if ir.contains(&&IrToken::CondType(CondType::Match)) {
            CondType::Match
        } else {
            return Err(RuleStructureError::NoConditionFocus);
        };

        let cond_ir = &mut ir.iter().copied();
        // takes all of the tokens before the input token and stores them in before
        // and discards the input token leaving cond_ir as the portion after it
        let before = &mut cond_ir.take_while(|&token| token != &IrToken::CondType(focus));

        Ok(CondPattern::new(
            focus,
            PatternList::new(ir_tokens_to_patterns(before, None, None, None)?),
            PatternList::new(ir_tokens_to_patterns(cond_ir, None, None, None)?),
        ))
}

/// Converts ir tokens to patterns
fn ir_tokens_to_patterns<'ir, 's: 'ir>(ir: &mut impl Iterator<Item = &'ir IrToken<'s>>, default_scope_ids: Option<&RefCell<DefaultScopeIds>>, parent_scope: Option<&ScopeId<'s>>, end_at: Option<ScopeType>) -> Result<Vec<Pattern<'s>>, RuleStructureError<'s>> {
    let mut patterns = Vec::new();

    while let Some(ir_token) = ir.next() {
        let pattern = match ir_token {
            IrToken::Phone(phone) => Pattern::new_phone(*phone),
            IrToken::Any => Pattern::new_any(any_id(default_scope_ids, parent_scope.cloned())),
            // starts a default labeled option scope
            IrToken::ScopeStart(ScopeType::Optional) => {
                let id = optional_id(default_scope_ids, parent_scope.cloned());

                let child_ids = default_scope_ids.map(|_| RefCell::default());

                Pattern::new_optional(ir_tokens_to_patterns(ir, child_ids.as_ref(), id.as_ref(), Some(ScopeType::Optional))?, id)
            },
            // starts a default labeled selection scope
            IrToken::ScopeStart(ScopeType::Selection) => {
                let id = selection_id(default_scope_ids, parent_scope.cloned());

                let child_ids = default_scope_ids.map(|_| RefCell::default());

                Pattern::new_selection(selection_contents_to_patterns(ir, child_ids.as_ref(), id.as_ref())?, id)
            },
            IrToken::ScopeStart(ScopeType::Repetition) => {
                let(inclusive, exclusive) = ir_to_repetition(ir)?;
                Pattern::new_repetition(None, inclusive, exclusive)
            },
            // ensures a label is proceeding a labelable token then creates that token with the label
            IrToken::Label(name) => {
                let next = ir.next();
                let id = Some(ScopeId::Name(name));

                if let Some(IrToken::ScopeStart(kind)) = next {
                    let child_ids = Some(&RefCell::default());

                    match kind {
                        ScopeType::Optional => Pattern::new_optional(ir_tokens_to_patterns(ir, child_ids, id.as_ref(), Some(ScopeType::Optional))?, id),
                        ScopeType::Selection => Pattern::new_selection(selection_contents_to_patterns(ir, child_ids, id.as_ref())?, id),
                        ScopeType::Repetition => {
                            let(inclusive, exclusive) = ir_to_repetition(ir)?;
                            Pattern::new_repetition(Some(*name), inclusive, exclusive)
                        },
                    }
                } else if let Some(IrToken::Any) = next {
                    Pattern::new_any(id)
                } else {
                    return Err(RuleStructureError::LabelNotFollowedByScope(name));
                }
            },
            // ends a scope returning either its contents or a related error
            IrToken::ScopeEnd(kind) => return if Some(*kind) == end_at {
                Ok(patterns)
            } else if let Some(start_type) = end_at {
                Err(RuleStructureError::MismatchedScopeBounds(start_type, *kind))
            } else {
                Err(RuleStructureError::UnopendScope(*kind))
            },
            IrToken::ArgSep => return Err(RuleStructureError::ArgSepOutOfSelection),
            IrToken::CondType(r#type) => return Err(RuleStructureError::UnexpectedCondType(*r#type)),
            IrToken::Negative if end_at == Some(ScopeType::Repetition) => {
                patterns.push(Pattern::List(PatternList::default())); // signals negative
                return Ok(patterns);
            }
            // these tokens should be removed in checking
            _ => return Err(RuleStructureError::UnexpectedToken(*ir_token)),
        };

        patterns.push(pattern);
    }

    Ok(patterns)
}

fn ir_to_repetition<'ir, 's: 'ir>(ir: &mut impl Iterator<Item = &'ir IrToken<'s>>) -> Result<(PatternList<'s>, Option<PatternList<'s>>), RuleStructureError<'s>> {
    let followed_by_exclusive = |pat: &Pattern<'_>| pat == &Pattern::List(PatternList::default());

    let mut inclusive_patterns = ir_tokens_to_patterns(ir, None, None, Some(ScopeType::Repetition))?;

    let has_exclusive = inclusive_patterns.pop_if(|pat| followed_by_exclusive(pat)).is_some();

    if inclusive_patterns.is_empty() {
        return Err(RuleStructureError::EmptyRepetition);
    }

    let exclusive = if has_exclusive {
        let exclusive_patterns = ir_tokens_to_patterns(ir, None, None, Some(ScopeType::Repetition))?;

        match exclusive_patterns.last() {
            None => return Err(RuleStructureError::EmptyExclusion),
            Some(pat) if followed_by_exclusive(pat) => return Err(RuleStructureError::UnexpectedToken(IrToken::Negative)),
            _ => (),
        }

        Some(PatternList::new(exclusive_patterns))
    } else {
        None
    };

    let inclusive = PatternList::new(inclusive_patterns);

    Ok((inclusive, exclusive))
}

/// Converts the ir tokens in a selection scope to a list of pattern lists
/// where each is an option to be selected by the scope: 
/// (options are seperated by the `ArgSep` token)
fn selection_contents_to_patterns<'ir, 's: 'ir>(ir: &mut impl Iterator<Item = &'ir IrToken<'s>>, default_scope_ids: Option<&RefCell<DefaultScopeIds>>, scope: Option<&ScopeId<'s>>) -> Result<Vec<Vec<Pattern<'s>>>, RuleStructureError<'s>> {
    let mut options = Vec::new();
    // scope_stack tracks which scope the function is analyzing to determine when to seperate options and return
    let mut scope_stack = Vec::new();

    // continues creates options until the scope's end is found
    'option_parser: loop {
        let mut option = Vec::new();

        // continously takes the next item in ir, if there is not another one an error is returned
        for ir_token in ir.by_ref() {
            match ir_token {
                // if there is an ArgSep token directly in the selection scope,
                // the option accumulator is pushed and a new one is started
                IrToken::ArgSep if scope_stack.is_empty() => {
                    options.push(option);
                    continue 'option_parser;
                },
                IrToken::ScopeEnd(kind) => {
                    // if the scope end is the end of the selection scope,
                    // the last option is pushed
                    // and the options are converted into pattern lists and returned
                    if scope_stack.is_empty() && kind == &ScopeType::Selection {
                        options.push(option);
                        let mut items = Vec::new();

                        for item in options {
                            items.push(ir_tokens_to_patterns(&mut item.into_iter(), default_scope_ids, scope, None)?);
                        }

                        return Ok(items);
                    } else if let Some(needed_end) = scope_stack.last() {
                        // otherwise the scope stack is popped or an error is returned
                        if needed_end == kind {
                            scope_stack.pop();
                            option.push(ir_token);
                        } else {
                            return Err(RuleStructureError::MismatchedScopeBounds(*needed_end, *kind));
                        }
                    } else {
                        return Err(RuleStructureError::MismatchedScopeBounds(ScopeType::Selection, ScopeType::Optional));
                    }
                },
                IrToken::ScopeStart(kind) => {
                    scope_stack.push(*kind);
                    option.push(ir_token);
                },
                _ => option.push(ir_token),
            }
        }
        
        return Err(RuleStructureError::UnclosedScope(ScopeType::Selection));
    }
}

/// Converts a line of ir into regions, all regions after the first are proceeded by a break
fn regionize_ir<'s, 'ir>(tokens: &'ir [IrToken<'s>]) -> (Vec<&'ir IrToken<'s>>, Vec<(Break, Vec<&'ir IrToken<'s>>)>) {
    let mut input_region = Vec::new();
    let mut other_regions = Vec::new();
    let mut after_input = false;

    for token in tokens {
        if let IrToken::Break(r#break) = token {
            other_regions.push((*r#break, Vec::new()));
            after_input = true;
        } else if after_input {
            // for after_input to be true, other_regions must have a length of at least one
            let last_index = other_regions.len() - 1;
            other_regions[last_index].1.push(token);
        } else {
            input_region.push(token);
        }
    }

    (input_region, other_regions)
}

/// Creates a default id for an optional scope and mutates the next default
fn optional_id<'s>(default_scope_ids: Option<&RefCell<DefaultScopeIds>>, parent: Option<ScopeId<'s>>) -> Option<ScopeId<'s>> {
    if let Some(ids) = default_scope_ids {
        let mut ids = ids.borrow_mut();
        let id_num = ids.optional;
        ids.optional += 1;
        Some(ScopeId::IOUnlabeled { parent: parent.map(Rc::new), id_num, label_type: LabelType::Scope(ScopeType::Optional) })
    } else {
        None
    }
}

/// Creates a default id for a selection scope and mutates the next default
fn selection_id<'s>(default_scope_ids: Option<&RefCell<DefaultScopeIds>>, parent: Option<ScopeId<'s>>) -> Option<ScopeId<'s>> {
    if let Some(ids) = default_scope_ids {
        let mut ids = ids.borrow_mut();
        let id_num = ids.selection;
        ids.selection += 1;
        Some(ScopeId::IOUnlabeled { parent: parent.map(Rc::new), id_num, label_type: LabelType::Scope(ScopeType::Selection) })
    } else {
        None
    }
}

/// Creates a default id for an an scope and mutates the next default
fn any_id<'s>(default_scope_ids: Option<&RefCell<DefaultScopeIds>>, parent: Option<ScopeId<'s>>) -> Option<ScopeId<'s>> {
    if let Some(ids) = default_scope_ids {
        let mut ids = ids.borrow_mut();
        let id_num = ids.any;
        ids.any += 1;
        Some(ScopeId::IOUnlabeled { parent: parent.map(Rc::new), id_num, label_type: LabelType::Any })
    } else {
        None
    }
}

/// An error that occurs when converting ir tokens to patterns
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum RuleStructureError<'s> {
    ArgSepOutOfSelection,
    BreakWithoutShift(Break),
    LabelNotFollowedByScope(&'s str),
    NoShift,
    UnclosedScope(ScopeType),
    UnopendScope(ScopeType),
    MismatchedScopeBounds(ScopeType, ScopeType),
    UnexpectedToken(IrToken<'s>),
    NoConditionFocus,
    AndDoesNotFollowCond(AndType),
    SecondShift(Shift),
    UnexpectedCondType(CondType),
    RepetitionOutOfCond,
    EmptyRepetition,
    EmptyExclusion,
}

impl std::error::Error for RuleStructureError<'_> {}

impl std::fmt::Display for RuleStructureError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArgSepOutOfSelection
                => write!(f, "Found '{}' outside of a selection scope ('{}')", IrToken::ArgSep, ScopeType::Selection),
            Self::BreakWithoutShift(r#break)
                => write!(f, "Found '{break}' without a proceeding shift token"),
            Self::LabelNotFollowedByScope(name)
                => write!(f, "Label '{}' is not followed by a scope", IrToken::Label(name)),
            Self::NoShift => write!(f, "Rule does not contains a shift token"),
            Self::UnopendScope(kind) => write!(f, "Found unopened '{}'", kind.fmt_end()),
            Self::UnclosedScope(kind) => write!(f, "Found unclosed '{}'", kind.fmt_start()),
            Self::MismatchedScopeBounds(start, end)
                => write!(f, "Found mismatched scope bounds '{}'...'{}'", start.fmt_start(), end.fmt_end()),
            Self::UnexpectedToken(ir_token) => write!(f, "Found unexpected token '{ir_token}'"),
            Self::NoConditionFocus => write!(f, "Found condition without an input patern ('{}') or equality ('{}')", CondType::Pattern, CondType::Match),
            Self::AndDoesNotFollowCond(and_type) => write!(f, "Found '{and_type}' outside of a condition"),
            Self::SecondShift(shift)
                => write!(f, "Found a second shift token '{shift}' after the first"),
            Self::UnexpectedCondType(r#type)
                => write!(f, "Found '{type}' either outside of a condition or after '{}' or '{}'", CondType::Pattern, CondType::Match),
            Self::RepetitionOutOfCond => write!(f, "Repetitions ('{}...{}') are not allowed outside of conditions and anti-conditions", ScopeType::Repetition.fmt_start(), ScopeType::Repetition.fmt_end()),
            Self::EmptyRepetition => write!(f, "A repetition must contain some inclusive pattern"),
            Self::EmptyExclusion => write!(f, "A repetition exclusion must contain some pattern"),
        }
    }
}