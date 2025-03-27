use crate::{meta_tokens::ScopeType, rules::conditions::CondType, tokens::COMMENT_LINE_START};
use super::{ir::{Break, IrToken}, IrLine};

#[cfg(test)]
mod tests;

/// Checks a list of token lines by line to ensure proper structure
/// 
/// Empty lines are considered ok
/// 
/// For more information see `check_scopes` and `check_breaks`
pub fn check_tokens<'s>(token_lines: &[IrLine<'s>]) -> Result<(), (IrStructureError<'s>, usize)> {
    let token_lines = token_lines
        .iter()
        .enumerate()
        .map(|(num, line)| (num + 1, line));

    for (line_num, line) in token_lines {
        check_token_line(line).map_err(|e| (e, line_num))?
    }

    Ok(())
}

/// Checks a line of tokens to ensure proper structure
/// 
/// Empty lines are considered ok
/// 
/// For more information see `check_scopes` and `check_breaks`
pub fn check_token_line<'s>(line: &IrLine<'s>) -> Result<(), IrStructureError<'s>> {
    if let IrLine::Ir(line) = line {
        if !line.is_empty() {
            check_scopes(line)?;
            check_breaks(line)?;
        }
    }
    Ok(())
}



/// Check to ensure that:
/// - there is exactally one shift break
/// - the shift break is proceeded by at least one phone
/// - conditions come after the shift
/// - no anti-condition proceed a condition
/// - no input patterns are outside of (anti-)conditions
/// - each (anti-)condition had exactally one input pattern
fn check_breaks<'s>(line: &[IrToken<'s>]) -> Result<(), IrStructureError<'s>> {
    let mut found_shift = false;
    let mut found_anti_conds = false;
    let mut in_cond = false;
    let mut inputs_in_cond = 0;

    for token in line {
        match token {
            IrToken::Break(break_type) => match break_type {
                Break::Shift(_) => if found_shift {
                    return Err(IrStructureError::ShiftAfterShift(*break_type))
                } else {
                    found_shift = true;
                },
                _ if !found_shift => {
                    return Err(IrStructureError::BreakBeforeShift(*break_type))
                },
                Break::AntiCond => {
                    found_anti_conds = true;
                    if in_cond {
                        if inputs_in_cond == 0 {
                            return Err(IrStructureError::NoFocusInCond)
                        } else if inputs_in_cond > 1 {
                            return Err(IrStructureError::ManyInputsInCond)
                        }
                    }

                    in_cond = true;
                    inputs_in_cond = 0;
                },
                Break::Cond if found_anti_conds => {
                    return Err(IrStructureError::AntiCondBeforeCond)
                },
                Break::Cond => {
                    if in_cond {
                        if inputs_in_cond == 0 {
                            return Err(IrStructureError::NoFocusInCond)
                        } else if inputs_in_cond > 1 {
                            return Err(IrStructureError::ManyInputsInCond)
                        }
                    }

                    in_cond = true;
                    inputs_in_cond = 0;
                },
            },
            IrToken::CondFocus(focus) => if !in_cond {
                return Err(IrStructureError::FocusOutOfCond(*focus));
            } else {
                inputs_in_cond += 1;
            }
            IrToken::Gap => if !in_cond {
                return Err(IrStructureError::GapOutOfCond);
            }
            _ => ()
        }
    }

    if in_cond {
        if inputs_in_cond == 0 {
            return Err(IrStructureError::NoFocusInCond)
        } else if inputs_in_cond > 1 {
            return Err(IrStructureError::ManyInputsInCond)
        }
    }

    if found_shift {
        Ok(())
    } else {
        Err(IrStructureError::NoShift)
    }
}

/// Check to ensure that:
/// - all scopes are properly opened and closed
/// - argument seperators are only in selection scopes
/// - scope labels only proceed scopes, anys, gaps
/// - only phones, scope labels, and scopes occur in scopes
///     (and argument seperators for selection scopes)
fn check_scopes<'s>(line: &[IrToken<'s>]) -> Result<(), IrStructureError<'s>> {
    let mut scope_stack = Vec::new();
    let mut selecting = None;

    for token in line {
        match token {
            IrToken::ScopeStart(kind) => {
                selecting = None;
                scope_stack.push(*kind)
            },
            IrToken::Any | IrToken::Gap => if selecting.is_some() {
                selecting = None;
            },
            _ if selecting.is_some() => {
                return Err(IrStructureError::SelectionDoesNotProceedScope(selecting.unwrap()))
            },
            IrToken::Phone(_) => (),
            IrToken::Label(name) => selecting = Some(name),
            IrToken::ScopeEnd(end_type) => {
                if let Some(start_type) = scope_stack.pop() {
                    if start_type != *end_type {
                        return Err(IrStructureError::MismatchedScopeBounds(start_type, *end_type));
                    }
                } else {
                    return Err(IrStructureError::UnopenedScope(*end_type));
                }
            },
            IrToken::ArgSep => if scope_stack.last() != Some(&ScopeType::Selection) {
                return Err(IrStructureError::MisplacedArgSep)
            },
            _ if scope_stack.is_empty() => (),
            _ => return Err(IrStructureError::DisallowedTokenInScope(*token))
        }
    }

    if let Some(kind) = scope_stack.pop() {
        Err(IrStructureError::UnclosedScope(kind))
    } else {
        Ok(())
    }
}

/// Errors that occur when checking the validity of ir tokens
/// indicating an invalid state
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum IrStructureError<'s> {
    UnopenedScope(ScopeType),
    UnclosedScope(ScopeType),
    MismatchedScopeBounds(ScopeType, ScopeType),
    MisplacedArgSep,
    DisallowedTokenInScope(IrToken<'s>),
    SelectionDoesNotProceedScope(&'s str),
    NoShift,
    ShiftAfterShift(Break),
    BreakBeforeShift(Break),
    AntiCondBeforeCond,
    NoFocusInCond,
    ManyInputsInCond,
    FocusOutOfCond(CondType),
    GapOutOfCond,
}

impl std::error::Error for IrStructureError<'_> {}

impl std::fmt::Display for IrStructureError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::UnopenedScope(kind) => {
                format!("Found scope end '{}' with no corresponding start '{}'", kind.fmt_end(), kind.fmt_start())
            },
            Self::UnclosedScope(kind) => {
                format!("Found scope start '{}' with no corresponding end '{}'", kind.fmt_start(), kind.fmt_end())
            },
            Self::MismatchedScopeBounds(start_type, end_type) => {
                format!("Found scope start '{}' closed with mismatched scope end '{}'", start_type.fmt_start(), end_type.fmt_end())
            },
            Self::MisplacedArgSep => {
                format!("Found a '{}' out side of a selection scope ('{}')", IrToken::ArgSep, ScopeType::Selection)
            },
            Self::DisallowedTokenInScope(token) => {
                format!("A '{token}' token may not occur in a scope")
            },
            Self::SelectionDoesNotProceedScope(name) => {
                format!("Scope label '{}' does not proceed a scope", IrToken::Label(name))
            },
            Self::AntiCondBeforeCond => {
                format!("Found anti-conditon (denoted '{}') before a condition (denoted '{}')", Break::AntiCond, Break::Cond)
            },
            Self::BreakBeforeShift(r#break) => {
                format!("Found token '{}' in the input of a sound change", r#break)
            },
            Self::NoShift => {
                format!("Found line with no shift token, consider commenting it out with '{COMMENT_LINE_START}'")
            },
            Self::ShiftAfterShift(shift) => {
                format!("Found shift token '{shift}' after another shift token")
            },
            Self::NoFocusInCond => {
                format!("Found condition or anti-condition without the input pattern ('{}') or equality operator ('{}')", CondType::MatchInput, CondType::Equality)
            },
            Self::ManyInputsInCond => {
                format!("Found condition or anti-condition with multiple input patterns ('{}') or equality operators ('{}')", CondType::MatchInput, CondType::Equality)
            },
            Self::FocusOutOfCond(focus) => {
                format!("Found condition focus ('{focus}') outside of a condition or anti-condition")
            },
            Self::GapOutOfCond => {
                format!("Found gap pattern ('{}') outside of a condition or anti-condition", IrToken::Gap)
            },
        };

        write!(f, "{}", s)
    }
}