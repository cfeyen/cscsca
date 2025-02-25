use crate::meta_tokens::{Direction, Shift, ShiftType};
use super::*;

#[test]
fn check_shift_only() {
    let shift = Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay });

    assert_eq!(
        Ok(()),
        check_tokens(&[IrLine::Ir(vec![IrToken::Break(shift)])])
    );
}

#[test]
fn check_basic_shift() {
    let shift = Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay });

    assert_eq!(
        Ok(()),
        check_tokens(&[IrLine::Ir(vec![IrToken::Phone("a"), IrToken::Break(shift), IrToken::Phone("b")])])
    );
}

#[test]
fn check_two_shifts() {
    let shift = Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay });

    assert_eq!(
        Err((IrStructureError::ShiftAfterShift(shift), 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::Phone("a"), IrToken::Break(shift), IrToken::Phone("b"), IrToken::Break(shift), IrToken::Phone("c")])])
    );
}

#[test]
fn check_cond_shift() {
    let shift = Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay });

    assert_eq!(
        Err((IrStructureError::BreakBeforeShift(Break::Cond), 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::Break(Break::Cond), IrToken::Break(shift),])])
    );
}

#[test]
fn check_shift_cond() {
    let shift = Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay });

    assert_eq!(
        Err((IrStructureError::NoInputInCond, 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::Break(shift), IrToken::Break(Break::Cond),])])
    );
}

#[test]
fn check_shift_cond_input() {
    let shift = Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay });

    assert_eq!(
        Ok(()),
        check_tokens(&[IrLine::Ir(vec![IrToken::Break(shift), IrToken::Break(Break::Cond), IrToken::Input])])
    );
}

#[test]
fn check_shift_cond_inputs() {
    let shift = Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay });

    assert_eq!(
        Err((IrStructureError::ManyInputsInCond, 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::Break(shift), IrToken::Break(Break::Cond), IrToken::Input, IrToken::Input])])
    );
}

#[test]
fn check_shift_anti_cond_cond() {
    let shift = Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay });

    assert!(
        // either of theses errors is acceptable here
        [Err((IrStructureError::NoInputInCond, 1)), Err((IrStructureError::AntiCondBeforeCond, 1))]
            .contains(&check_tokens(&[IrLine::Ir(vec![IrToken::Break(shift), IrToken::Break(Break::AntiCond), IrToken::Break(Break::Cond)])]))
    );
}

#[test]
fn check_shift_anti_cond_input_cond_input() {
    let shift = Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay });

    assert_eq!(
        Err((IrStructureError::AntiCondBeforeCond, 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::Break(shift), IrToken::Break(Break::AntiCond), IrToken::Input, IrToken::Break(Break::Cond), IrToken::Input])])
    );
}

#[test]
fn check_shift_anti_cond_input() {
    let shift = Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay });

    assert_eq!(
        Ok(()),
        check_tokens(&[IrLine::Ir(vec![IrToken::Break(shift), IrToken::Break(Break::AntiCond), IrToken::Input])])
    );
}

#[test]
fn check_shift_anti_cond() {
    let shift = Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay });

    assert_eq!(
        Err((IrStructureError::NoInputInCond, 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::Break(shift), IrToken::Break(Break::AntiCond)])])
    );
}

#[test]
fn check_nothing() {
    assert_eq!(
        Ok(()),
        check_tokens(&[])
    );
}

#[test]
fn check_nothings() {
    assert_eq!(
        Ok(()),
        check_tokens(&[IrLine::Empty, IrLine::Ir(Vec::new()), IrLine::Empty])
    );
}

#[test]
fn check_scope_start() {
    assert_eq!(
        Err((IrStructureError::UnclosedScope(ScopeType::Selection), 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::ScopeStart(ScopeType::Selection)])])
    );
}

#[test]
fn check_scope_end() {
    assert_eq!(
        Err((IrStructureError::UnopenedScope(ScopeType::Optional), 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::ScopeEnd(ScopeType::Optional)])])
    );
}

#[test]
fn check_empty_scope() {
    assert_eq!(
        Err((IrStructureError::NoShift, 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::ScopeStart(ScopeType::Optional), IrToken::ScopeEnd(ScopeType::Optional)])])
    );
}

#[test]
fn check_mismatched_scope() {
    assert_eq!(
        Err((IrStructureError::MismatchedScopeBounds(ScopeType::Selection, ScopeType::Optional), 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::ScopeStart(ScopeType::Selection), IrToken::ScopeEnd(ScopeType::Optional)])])
    );
}

#[test]
fn check_nested_scopes() {
    assert_eq!(
        Err((IrStructureError::NoShift, 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::ScopeStart(ScopeType::Selection), IrToken::ScopeStart(ScopeType::Optional), IrToken::ScopeEnd(ScopeType::Optional), IrToken::ScopeEnd(ScopeType::Selection)])])
    );
}

#[test]
fn check_adjacent_scopes() {
    assert_eq!(
        Err((IrStructureError::NoShift, 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::ScopeStart(ScopeType::Selection), IrToken::ScopeEnd(ScopeType::Selection), IrToken::ScopeStart(ScopeType::Optional), IrToken::ScopeEnd(ScopeType::Optional)])])
    );
}

#[test]
fn check_overlaping_scopes() {
    assert_eq!(
        Err((IrStructureError::MismatchedScopeBounds(ScopeType::Optional, ScopeType::Selection), 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::ScopeStart(ScopeType::Selection), IrToken::ScopeStart(ScopeType::Optional), IrToken::ScopeEnd(ScopeType::Selection), IrToken::ScopeEnd(ScopeType::Optional)])])
    );
}

#[test]
fn check_shift_in_scope() {
    let shift = Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay });

    assert_eq!(
        Err((IrStructureError::DisallowedTokenInScope(IrToken::Break(shift)), 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::ScopeStart(ScopeType::Selection), IrToken::Break(shift), IrToken::ScopeEnd(ScopeType::Selection)])])
    );
}

#[test]
fn check_shift_between_scopes() {
    let shift = Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay });

    assert_eq!(
        Ok(()),
        check_tokens(&[IrLine::Ir(vec![IrToken::ScopeStart(ScopeType::Selection), IrToken::ScopeEnd(ScopeType::Selection), IrToken::Break(shift), IrToken::ScopeStart(ScopeType::Selection), IrToken::ScopeEnd(ScopeType::Selection)])])
    );
}

#[test]
fn check_arg_sep() {
    assert_eq!(
        Err((IrStructureError::MisplacedArgSep, 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::ArgSep])])
    );
}

#[test]
fn check_arg_sep_in_optional() {
    assert_eq!(
        Err((IrStructureError::MisplacedArgSep, 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::ScopeStart(ScopeType::Optional), IrToken::ArgSep, IrToken::ScopeEnd(ScopeType::Optional)])])
    );
}

#[test]
fn check_arg_sep_in_selection() {
    assert_eq!(
        Err((IrStructureError::NoShift, 1)),
        check_tokens(&[IrLine::Ir(vec![IrToken::ScopeStart(ScopeType::Selection), IrToken::ArgSep, IrToken::ScopeEnd(ScopeType::Selection)])])
    );
}

#[test]
fn check_any() {
    assert_eq!(Ok(()), check_token_line(&IrLine::Ir(vec![
        IrToken::Any,
        IrToken::Break(Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Move })),
    ])));
}

#[test]
fn check_labeled_any() {
    assert_eq!(Ok(()), check_token_line(&IrLine::Ir(vec![
        IrToken::Label("label"),
        IrToken::Any,
        IrToken::Break(Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Move })),
    ])));
}

#[test]
fn check_gap() {
    assert_eq!(Err(IrStructureError::GapOutOfCond), check_token_line(&IrLine::Ir(vec![
        IrToken::Break(Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Move })),
        IrToken::Gap,
    ])));
}

#[test]
fn check_gap_in_cond() {
    assert_eq!(Ok(()), check_token_line(&IrLine::Ir(vec![
        IrToken::Break(Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Move })),
        IrToken::Break(Break::Cond),
        IrToken::Gap,
        IrToken::Input,
    ])));
}