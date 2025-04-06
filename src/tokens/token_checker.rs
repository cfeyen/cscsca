use crate::{meta_tokens::{ScopeType, Shift}, rules::conditions::CondType, tokens::COMMENT_LINE_START};
use super::{ir::{Break, IrToken}, IrLine};

#[cfg(test)]
mod tests;

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

/// Converts a line of ir into regions where each (except the first) starts with a break
/// 
/// ## Invariant
/// - The first Break is None and all others are Some
/// - There is always at least one region
// ! The invariants must be upheld
pub fn regionize_ir<'s, 'a>(tokens: &'a [IrToken<'s>]) -> Vec<(Option<Break>, Vec<&'a IrToken<'s>>)> {
    let mut regions = vec![(None, Vec::new())];

    for token in tokens {
        if let IrToken::Break(r#break) = token {
            // Only the first region should have None break
            regions.push((Some(*r#break), Vec::new())) 
        } else {
            let last_index = regions.len() - 1;
            regions[last_index].1.push(token);
        }
    }

    regions
}



/// Check to ensure that:
/// - there is exactally one shift break
/// - conditions come after the shift
/// - no anti-condition procees a condition
/// - there is no condition focus outside of a(n) (anti-)conditions
/// - each (anti-)condition had exactally one focus
/// - gaps do not occur out of (anti-)conditions
fn check_breaks<'s>(line: &[IrToken<'s>]) -> Result<(), IrStructureError<'s>> {
    let regions = regionize_ir(line);

    // ensures that the second region starts with a shift
    if let Some((r#break, _)) = regions.get(1) {
        if !matches!(r#break, Some(Break::Shift(_))) {
            return if regions.iter()
                .filter(|(r#break, _)| matches!(r#break, Some(Break::Shift(_))))
                .count() > 0 {
                    // returns if there is a break
                    Err(IrStructureError::BreakBeforeShift(r#break.expect("only the first region should have None break")))
            } else {
                Err(IrStructureError::NoShift)
            };
        }
    } else {
        return Err(IrStructureError::NoShift);
    }

    let mut found_conds = false;
    let mut found_anti_conds = false;

    // otherwise check break order
    if let Some(conds) = regions.get(2..) {
        for region in conds {
            match region.0 {
                Some(Break::Shift(shift)) => Err(IrStructureError::ShiftAfterShift(shift)),
                Some(Break::Cond) if found_anti_conds => Err(IrStructureError::AntiCondBeforeCond),
                Some(Break::Cond) => {
                    found_conds = true;
                    Ok(())
                },
                Some(Break::AntiCond) => {
                    found_conds = true;
                    found_anti_conds = true;
                    Ok(())
                },
                Some(Break::And) if found_conds => Ok(()),
                Some(Break::And) => Err(IrStructureError::AndOutOfCond),
                None => panic!("There should be no region after the first will a None break"),
            }?;
        }
    }

    // check that condition foci are valid in regions
    for (r#break, tokens) in regions {
        // filters out foci
        let mut foci = tokens
            .iter()
            .filter(|t| matches!(t, IrToken::CondType(_)));

        match r#break {
            None | Some(Break::Shift(_)) => if let Some(IrToken::CondType(focus)) = foci.next() {
                return Err(IrStructureError::FocusOutOfCond(*focus));
            } else if tokens.contains(&&IrToken::Gap) {
                return Err(IrStructureError::GapOutOfCond)
            },
            _ => {
                let foci = foci.count();
                if foci == 0 {
                    return Err(IrStructureError::NoFocusInCond);
                } else if foci > 1 {
                    return Err(IrStructureError::ManyFociInCond);
                }
            }
        }
    }

    Ok(())
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
    ShiftAfterShift(Shift),
    BreakBeforeShift(Break),
    AntiCondBeforeCond,
    NoFocusInCond,
    ManyFociInCond,
    FocusOutOfCond(CondType),
    GapOutOfCond,
    AndOutOfCond,
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
                format!("Found rule with no shift token, consider commenting it out with '{COMMENT_LINE_START}'")
            },
            Self::ShiftAfterShift(shift) => {
                format!("Found shift token '{shift}' after another shift token")
            },
            Self::NoFocusInCond => {
                format!("Found condition or anti-condition without the input pattern ('{}') or equality operator ('{}')", CondType::MatchInput, CondType::Equality)
            },
            Self::ManyFociInCond => {
                format!("Found condition or anti-condition with multiple input patterns ('{}') or equality operators ('{}')", CondType::MatchInput, CondType::Equality)
            },
            Self::FocusOutOfCond(focus) => {
                format!("Found condition focus ('{focus}') outside of a condition or anti-condition")
            },
            Self::GapOutOfCond => {
                format!("Found gap pattern ('{}') outside of a condition or anti-condition", IrToken::Gap)
            },
            Self::AndOutOfCond => {
                format!("Found conditional and ('{}') outside of a condition or anti-condition", Break::And)
            }
        };

        write!(f, "{}", s)
    }
}

/// Checks a list of token lines by line to ensure proper structure
/// 
/// Empty lines are considered ok
/// 
/// For more information see `check_scopes` and `check_breaks`
#[cfg(test)]
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