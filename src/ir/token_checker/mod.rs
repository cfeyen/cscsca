use crate::{tokens::{ScopeType, Shift}, rules::conditions::CondType, ir::COMMENT_LINE_START};
use super::{tokens::{Break, IrToken}, IrLine};

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

/// Converts a line of ir into regions, all regions after the first are proceeded by a break
pub fn regionize_ir<'s, 'ir>(tokens: &'ir [IrToken<'s>]) -> (Vec<&'ir IrToken<'s>>, Vec<(Break, Vec<&'ir IrToken<'s>>)>) {
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



/// Check to ensure that:
/// - there is exactally one shift break
/// - conditions come after the shift
/// - no anti-condition procees a condition
/// - there is no condition focus outside of a(n) (anti-)conditions
/// - each (anti-)condition had exactally one focus
/// - gaps do not occur out of (anti-)conditions
fn check_breaks<'s>(line: &[IrToken<'s>]) -> Result<(), IrStructureError<'s>> {
    let (_, regions) = regionize_ir(line);

    // ensures that the second region starts with a shift
    if let Some((r#break, _)) = regions.first() {
        if !matches!(r#break, Break::Shift(_)) {
            return if regions.iter()
                .filter(|(r#break, _)| matches!(r#break, Break::Shift(_)))
                .count() > 0 {
                    // returns if there is a break
                    Err(IrStructureError::BreakBeforeShift(*r#break))
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
    if let Some(conds) = regions.get(1..) {
        for region in conds {
            match region.0 {
                Break::Shift(shift) => Err(IrStructureError::ShiftAfterShift(shift)),
                Break::Cond if found_anti_conds => Err(IrStructureError::AntiCondBeforeCond),
                Break::Cond => {
                    found_conds = true;
                    Ok(())
                },
                Break::AntiCond => {
                    found_conds = true;
                    found_anti_conds = true;
                    Ok(())
                },
                Break::And if found_conds => Ok(()),
                Break::And => Err(IrStructureError::AndOutOfCond),
            }?;
        }
    }

    // check that condition foci are valid in regions
    for (r#break, tokens) in regions {
        // filters out foci
        let mut foci = tokens
            .iter()
            .filter(|t| matches!(t, IrToken::CondType(_)));

        if let Break::Shift(_) = r#break {
            if let Some(IrToken::CondType(focus)) = foci.next() {
                return Err(IrStructureError::FocusOutOfCond(*focus));
            } else if tokens.contains(&&IrToken::Gap) {
                return Err(IrStructureError::GapOutOfCond);
            }
        } else {
            let foci = foci.count();
            if foci == 0 {
                return Err(IrStructureError::NoFocusInCond);
            } else if foci > 1 {
                return Err(IrStructureError::ManyFociInCond);
            }
        }
    }

    Ok(())
}

/// Check to ensure that:
/// - all scopes are properly opened and closed
/// - argument seperators are only in selection scopes
/// - scope labels only proceed labelable tokens
/// - only scope-valid characters occur in scopes
///   (and argument seperators for selection scopes)
fn check_scopes<'s>(line: &[IrToken<'s>]) -> Result<(), IrStructureError<'s>> {
    let mut scope_stack = Vec::new();
    let mut label = None;

    for token in line {
        if !(scope_stack.is_empty() || token.valid_in_scope()) {
            return Err(IrStructureError::DisallowedTokenInScope(*token));
        }

        match label {
            Some(name) if !token.labelable() =>
                return Err(IrStructureError::LabelProceedsUnlabelable(name, *token)),
            _ => ()
        }

        match token {
            IrToken::ScopeStart(kind) => {
                label = None;
                scope_stack.push(*kind);
            },
            IrToken::Any | IrToken::Gap => label = None,
            IrToken::Label(name) => label = Some(name),
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
            _ => ()
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
    LabelProceedsUnlabelable(&'s str, IrToken<'s>),
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
            Self::LabelProceedsUnlabelable(name, token) => {
                format!("Label '{}' proceeds unlabelable token '{token}'", IrToken::Label(name))
            },
            Self::AntiCondBeforeCond => {
                format!("Found anti-conditon (denoted '{}') before a condition (denoted '{}')", Break::AntiCond, Break::Cond)
            },
            Self::BreakBeforeShift(r#break) => {
                format!("Found token '{break}' in the input of a sound change")
            },
            Self::NoShift => {
                format!("Found rule with no shift token, consider commenting it out with '{COMMENT_LINE_START}'")
            },
            Self::ShiftAfterShift(shift) => {
                format!("Found shift token '{shift}' after another shift token")
            },
            Self::NoFocusInCond => {
                format!("Found condition or anti-condition without the input pattern ('{}') or equality operator ('{}')", CondType::Pattern, CondType::Match)
            },
            Self::ManyFociInCond => {
                format!("Found condition or anti-condition with multiple input patterns ('{}') or equality operators ('{}')", CondType::Pattern, CondType::Match)
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

        write!(f, "{s}")
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