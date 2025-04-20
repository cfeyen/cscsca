use std::{cell::RefCell, rc::Rc};

use conditions::{Cond, CondType};
use sound_change_rule::SoundChangeRule;
use tokens::{LabelType, RuleToken, ScopeId};

use crate::{ir::{tokens::{Break, IrToken, AND_CHAR}, IrLine}, tokens::{ScopeType, Shift}};

pub mod sound_change_rule;
pub mod conditions;
pub mod tokens;

#[cfg(test)]
mod tests;

/// A rule, executed command, or nothing representing a line of source code
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum RuleLine<'s> {
    Rule(SoundChangeRule<'s>),
    Cmd,
    Empty,
}

/// Default ids for unlabled scopes
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct DefaultScopeIds {
    optional: usize,
    selection: usize,
    any: usize,
}

/// Builds a sound change rule out of a line of ir tokens
pub fn build_rule<'s>(line: &IrLine<'s>) -> Result<RuleLine<'s>, RuleStructureError<'s>> {
    let line = match line {
        IrLine::Empty => return Ok(RuleLine::Empty),
        IrLine::Cmd(_cmd, _args) => return Ok(RuleLine::Cmd),
        IrLine::Ir(tokens) if tokens.is_empty() => return Ok(RuleLine::Empty),
        IrLine::Ir(tokens) => tokens
    };

    let (input_region, other_regions) = regionize_ir(line);
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
            Break::And => {
                let mut cond = ir_to_cond(&tokens)?;

                let last_cond = if to_anti_conds {
                    &mut anti_conds
                } else {
                    &mut conds
                }
                .last_mut()
                .ok_or(RuleStructureError::AndDoesNotFollowCond)?;

                cond.set_and(std::mem::take(last_cond));
                *last_cond = cond;
            }
        }
    }

    Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input,
        output,
        conds: if conds.is_empty() { vec![Cond::default()] } else { conds },
        anti_conds,
    }))
}

/// Converts the ir tokens for the input and output of a rule to rule tokens
#[inline]
fn ir_to_input_output<'s>(ir: &[&IrToken<'s>]) -> Result<Vec<RuleToken<'s>>, RuleStructureError<'s>> {
    ir_tokens_to_rule_tokens(
        &mut ir.iter().copied(), 
        Some(&RefCell::default()),
        None, 
        None
    )
}

/// Converts lists of ir tokens for the (anti-)conditions of a rule to a list of Cond structs
fn ir_to_cond<'s>(ir: &[&IrToken<'s>]) -> Result<Cond<'s>, RuleStructureError<'s>> {
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

        Ok(Cond::new(
            focus,
            ir_tokens_to_rule_tokens(before, None, None, None)?,
            ir_tokens_to_rule_tokens(cond_ir, None, None, None)?,
        ))
}

/// Converts ir tokens to rule tokens
fn ir_tokens_to_rule_tokens<'ir, 's: 'ir>(ir: &mut impl Iterator<Item = &'ir IrToken<'s>>, default_scope_ids: Option<&RefCell<DefaultScopeIds>>, parent_scope: Option<&ScopeId<'s>>, end_at: Option<ScopeType>) -> Result<Vec<RuleToken<'s>>, RuleStructureError<'s>> {
    let mut rule_tokens = Vec::new();

    while let Some(ir_token) = ir.next() {
        let rule_token = match ir_token {
            IrToken::Phone(phone) => RuleToken::Phone(*phone),
            IrToken::Any => RuleToken::Any { id: any_id(default_scope_ids, parent_scope.cloned()) },
            IrToken::Gap => RuleToken::Gap { id: None },
            // starts a default labeled option scope
            IrToken::ScopeStart(ScopeType::Optional) => {
                let id = optional_id(default_scope_ids, parent_scope.cloned());

                RuleToken::OptionalScope {
                    content: ir_tokens_to_rule_tokens(ir, default_scope_ids, id.as_ref(), Some(ScopeType::Optional))?,
                    id,
                }
            },
            // starts a default labeled selection scope
            IrToken::ScopeStart(ScopeType::Selection) => {
                let id = selection_id(default_scope_ids, parent_scope.cloned());

                RuleToken::SelectionScope {
                    options: selection_contents_to_rule_tokens(ir, default_scope_ids, id.as_ref())?,
                    id,
                }
            },
            // ensures a label is proceeding a labelable token then creates that token with the label
            IrToken::Label(name) => {
                let next = ir.next();
                let id = Some(ScopeId::Name(name));

                if let Some(IrToken::ScopeStart(kind)) = next {
                    match kind {
                        ScopeType::Optional => RuleToken::OptionalScope {
                            content: ir_tokens_to_rule_tokens(ir, default_scope_ids, id.as_ref(), Some(ScopeType::Optional))?,
                            id,
                        },
                        ScopeType::Selection => RuleToken::SelectionScope {
                            options: selection_contents_to_rule_tokens(ir, default_scope_ids, id.as_ref())?,
                            id,
                        }
                    }
                } else if let Some(IrToken::Any) = next {
                    RuleToken::Any { id }
                } else if let Some(IrToken::Gap) = next {
                    RuleToken::Gap { id: Some(name) }
                } else {
                    return Err(RuleStructureError::LabelNotFollowedByScope(name));
                }
            },
            // ends a scope returning either its contents or a related error
            IrToken::ScopeEnd(kind) => return if Some(*kind) == end_at {
                Ok(rule_tokens)
            } else if let Some(start_type) = end_at {
                Err(RuleStructureError::MismatchedScopeBounds(start_type, *kind))
            } else {
                Err(RuleStructureError::UnopendScope(*kind))
            },
            IrToken::ArgSep => return Err(RuleStructureError::ArgSepOutOfSelection),
            IrToken::CondType(r#type) => return Err(RuleStructureError::UnexpectedCondType(*r#type)),
            // these tokens should be removed in checking
            _ => return Err(RuleStructureError::UnexpectedToken(*ir_token)),
        };

        rule_tokens.push(rule_token);
    }

    Ok(rule_tokens)
}

/// Converts the ir tokens in a selection scope to a list of rule token lists
/// where each is an option to be selected by the scope: 
/// (options are seperated by the `ArgSep` token)
fn selection_contents_to_rule_tokens<'ir, 's: 'ir>(ir: &mut impl Iterator<Item = &'ir IrToken<'s>>, default_scope_ids: Option<&RefCell<DefaultScopeIds>>, scope: Option<&ScopeId<'s>>) -> Result<Vec<Vec<RuleToken<'s>>>, RuleStructureError<'s>> {
    let mut options = Vec::new();
    // scope_stack tracks which scope the function is analyzing to determine when to seperate options and return
    let mut scope_stack = Vec::new();

    // continues creates options until a value is returned
    loop {
        let mut option = Vec::new();

        // continously takes the next item in ir, if there is not another one an error is returned
        // (uses loop { if let {} else {}} instead of while let {} so an error can be returned if there are no items left)
        loop {
            if let Some(ir_token) = ir.next() {
                match ir_token {
                    // if there is an ArgSep token directly in the selection scope,
                    // the option accumulator is pushed and a new one is started
                    IrToken::ArgSep if scope_stack.is_empty() => {
                        options.push(option);
                        break;
                    },
                    IrToken::ScopeEnd(kind) => {
                        // if the scope end is the end of the selection scope,
                        // the last option is pushed
                        // and the options are converted into rule token lists and returned
                        if scope_stack.is_empty() && kind == &ScopeType::Selection {
                            options.push(option);
                            let mut items = Vec::new();

                            for item in options {
                                items.push(ir_tokens_to_rule_tokens(&mut item.into_iter(), default_scope_ids, scope, None)?);
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
            } else {
                return Err(RuleStructureError::UnclosedScope(ScopeType::Selection))
            }
        }
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

/// An error that occurs when converting ir tokens to rule tokens
/// 
/// Some of these errors are duplicates of `TokenStructureError`s
/// that should be caught when the ir is checked
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
    AndDoesNotFollowCond,
    SecondShift(Shift),
    UnexpectedCondType(CondType)
}

impl std::error::Error for RuleStructureError<'_> {}

impl std::fmt::Display for RuleStructureError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::ArgSepOutOfSelection
                => format!("Found '{}' outside of a selection scope ('{}')", IrToken::ArgSep, ScopeType::Selection),
            Self::BreakWithoutShift(r#break)
                => format!("Found '{break}' without a proceeding shift token"),
            Self::LabelNotFollowedByScope(name)
                => format!("Label '{}' is not followed by a scope", IrToken::Label(name)),
            Self::NoShift => "Rule does not contains a shift token".to_string(),
            Self::UnopendScope(kind) => format!("Found unopened '{}'", kind.fmt_end()),
            Self::UnclosedScope(kind) => format!("Found unclosed '{}'", kind.fmt_start()),
            Self::MismatchedScopeBounds(start, end)
                => format!("Found mismatched scope bounds '{}'..'{}'", start.fmt_start(), end.fmt_end()),
            Self::UnexpectedToken(ir_token) => format!("Found unexpected token '{ir_token}'"),
            Self::NoConditionFocus => format!("Found condition without an input patern ('{}') or equality ('{}')", CondType::Pattern, CondType::Match),
            Self::AndDoesNotFollowCond => format!("Found '{AND_CHAR}' outside of a condition"),
            Self::SecondShift(shift)
                => format!("Found a second shift token '{shift}' after the first"),
            Self::UnexpectedCondType(r#type)
                => format!("Found '{type}' either outside of a condition or after '{}' or '{}'", CondType::Pattern, CondType::Match),
        };

        write!(f, "{s}")
    }
}