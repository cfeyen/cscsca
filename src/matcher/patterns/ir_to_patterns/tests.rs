use std::{num::NonZero, rc::Rc};

use crate::{phones::Phone, tokens::{Direction, Shift, ShiftType, AndType}};
use super::*;

const ONE: NonZero<usize> = NonZero::new(1).expect("1 ought to be nonzero");

/// Builds a sound change rules out of lines of ir tokens,
/// if there is an error it is returned with its line number
#[cfg(test)]
fn build_rules<'s>(token_lines: Vec<IrLine<'s>>) -> Result<Vec<RuleLine<'s>>, (RuleStructureError<'s>, usize)> {
    token_lines
        .into_iter()
        .enumerate()
        .map(|(num, line)| build_rule(line).map_err(|e| (e, num + 1)))
        .collect()
}

#[test]
fn no_rule() {
    assert_eq!(Ok(Vec::new()), build_rules(Vec::new()));
}

#[test]
fn empty_line() {
    assert_eq!(Ok(RuleLine::Empty), build_rule(IrLine::Empty));
    assert_eq!(Ok(RuleLine::Empty), build_rule(IrLine::Ir { tokens: Vec::new(), lines: ONE }));
}

#[test]
fn one_to_one() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: vec![Pattern::new_phone(Phone::Symbol("b"))],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            Vec::new(),
            Vec::new(),
        ).expect("pattern construction should be valid")),
    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone(Phone::Symbol("b")),
    ], lines: ONE }));
}

#[test]
fn three_to_three() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: vec![Pattern::new_phone(Phone::Symbol("d")), Pattern::new_phone(Phone::Symbol("e")), Pattern::new_phone(Phone::Symbol("f"))],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a")), Pattern::new_phone(Phone::Symbol("b")), Pattern::new_phone(Phone::Symbol("c"))]),
            Vec::new(),
            Vec::new(),
        ).expect("pattern construction should be valid")),
    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::Phone(Phone::Symbol("c")),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone(Phone::Symbol("d")),
        IrToken::Phone(Phone::Symbol("e")),
        IrToken::Phone(Phone::Symbol("f")),
    ], lines: ONE }));
}

#[test]
fn selected_three_to_selected_three() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    let input = PatternList::new(vec![Pattern::new_selection(
        vec![
            vec![Pattern::new_phone(Phone::Symbol("a"))],
            vec![Pattern::new_phone(Phone::Symbol("b"))],
            vec![Pattern::new_phone(Phone::Symbol("c"))],
        ],
        Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) }),
    )]);

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: vec![Pattern::new_selection(
            vec![
                vec![Pattern::new_phone(Phone::Symbol("d"))],
                vec![Pattern::new_phone(Phone::Symbol("e"))],
                vec![Pattern::new_phone(Phone::Symbol("f"))],
            ],
            Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) }),
        )],
        pattern: RefCell::new(
            RulePattern::new(input,
            Vec::new(),
            Vec::new(),
        ).expect("pattern construction should be valid")),

    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::ArgSep,
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::ArgSep,
        IrToken::Phone(Phone::Symbol("c")),
        IrToken::ScopeEnd(ScopeType::Selection),
        IrToken::Break(Break::Shift(shift)),
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone(Phone::Symbol("d")),
        IrToken::ArgSep,
        IrToken::Phone(Phone::Symbol("e")),
        IrToken::ArgSep,
        IrToken::Phone(Phone::Symbol("f")),
        IrToken::ScopeEnd(ScopeType::Selection),
    ], lines: ONE }));
}

#[test]
fn labeled_selected_three_to_selected_three() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    let input = PatternList::new(vec![Pattern::new_selection(
        vec![
            vec![Pattern::new_phone(Phone::Symbol("a"))],
            vec![Pattern::new_phone(Phone::Symbol("b"))],
            vec![Pattern::new_phone(Phone::Symbol("c"))],
        ],
        Some(ScopeId::Name("label")),
    )]);

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: vec![Pattern::new_selection(
            vec![
                vec![Pattern::new_phone(Phone::Symbol("d"))],
                vec![Pattern::new_phone(Phone::Symbol("e"))],
                vec![Pattern::new_phone(Phone::Symbol("f"))],
            ],
            Some(ScopeId::Name("label")),
        )],
        pattern: RefCell::new(
            RulePattern::new(input,
            Vec::new(),
            Vec::new(),
        ).expect("pattern construction should be valid")),

    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::Label("label"),
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::ArgSep,
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::ArgSep,
        IrToken::Phone(Phone::Symbol("c")),
        IrToken::ScopeEnd(ScopeType::Selection),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Label("label"),
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone(Phone::Symbol("d")),
        IrToken::ArgSep,
        IrToken::Phone(Phone::Symbol("e")),
        IrToken::ArgSep,
        IrToken::Phone(Phone::Symbol("f")),
        IrToken::ScopeEnd(ScopeType::Selection),
    ], lines: ONE }));
}

#[test]
fn labeled_phone_to_phone() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(
        Err((RuleStructureError::LabelNotFollowedByScope("label"), 1)),
        build_rules(vec![IrLine::Ir { tokens: vec![
            IrToken::Label("label"),
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Phone(Phone::Symbol("b")),
        ], lines: ONE }])
    );
}

#[test]
fn no_shift() {
    assert_eq!(
        Err((RuleStructureError::NoShift, 1)),
        build_rules(vec![IrLine::Ir { tokens: vec![
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::Phone(Phone::Symbol("c")),
        ], lines: ONE }])
    );
}

#[test]
fn no_output() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: Vec::new(),
        pattern: RefCell::new(
            RulePattern::new(PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            Vec::new(),
            Vec::new(),
        ).expect("pattern construction should be valid")),
    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Break(Break::Shift(shift)),
    ], lines: ONE }));
}

#[test]
fn single_option() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    let input = PatternList::new(vec![Pattern::new_optional(vec![Pattern::new_phone(Phone::Symbol("a"))], Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Optional) }))]);

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: Vec::new(),
        pattern: RefCell::new(
            RulePattern::new(input,
            Vec::new(),
            Vec::new(),
        ).expect("pattern construction should be valid")),
    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::ScopeStart(ScopeType::Optional),
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::ScopeEnd(ScopeType::Optional),
        IrToken::Break(Break::Shift(shift)),
    ], lines: ONE }));
}

#[test]
fn nested_scopes() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    let label_0 = Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) });
    let label_1 = Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Scope(ScopeType::Optional), parent: Some(Rc::new(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) })) });
    let label_2 = Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Scope(ScopeType::Optional), parent: Some(Rc::new(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) })) });

    let input = PatternList::new(vec![Pattern::new_selection(
        vec![
            vec![Pattern::new_phone(Phone::Symbol("a"))],
            vec![Pattern::new_optional(vec![Pattern::new_phone(Phone::Symbol("b"))], label_1.clone())],
            vec![Pattern::new_optional(vec![Pattern::new_phone(Phone::Symbol("c"))], label_2.clone())],
            vec![Pattern::new_phone(Phone::Symbol("d"))],
        ],
        label_0.clone()
    )]);

    assert_eq!(
        Ok(RuleLine::Rule { rule: SoundChangeRule {
            kind: shift,
            output: vec![Pattern::new_selection(
                vec![
                    vec![Pattern::new_phone(Phone::Symbol("e"))],
                    vec![Pattern::new_optional(vec![Pattern::new_phone(Phone::Symbol("f"))], label_1)],
                    vec![Pattern::new_optional(vec![Pattern::new_phone(Phone::Symbol("g"))], label_2)],
                    vec![Pattern::new_phone(Phone::Symbol("h"))],
                ],
                label_0
            )],
            pattern: RefCell::new(
            RulePattern::new(    input,
                Vec::new(),
                Vec::new(),
            ).expect("pattern construction should be valid")),
        }, lines: ONE }),
        build_rule(IrLine::Ir { tokens: vec![
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::ArgSep,
            IrToken::ScopeStart(ScopeType::Optional),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::ScopeEnd(ScopeType::Optional),
            IrToken::ArgSep,
            IrToken::ScopeStart(ScopeType::Optional),
            IrToken::Phone(Phone::Symbol("c")),
            IrToken::ScopeEnd(ScopeType::Optional),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("d")),
            IrToken::ScopeEnd(ScopeType::Selection),
            IrToken::Break(Break::Shift(shift)),
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone(Phone::Symbol("e")),
            IrToken::ArgSep,
            IrToken::ScopeStart(ScopeType::Optional),
            IrToken::Phone(Phone::Symbol("f")),
            IrToken::ScopeEnd(ScopeType::Optional),
            IrToken::ArgSep,
            IrToken::ScopeStart(ScopeType::Optional),
            IrToken::Phone(Phone::Symbol("g")),
            IrToken::ScopeEnd(ScopeType::Optional),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("h")),
            IrToken::ScopeEnd(ScopeType::Selection),
        ], lines: ONE })
    );
}

#[test]
fn single_cond() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: vec![Pattern::new_phone(Phone::Symbol("b"))],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            vec![CondPattern::new(
                CondType::Pattern,
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("c"))]),
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("d"))]),
            )],
            Vec::new(),
        ).expect("pattern construction should be valid")),
    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::Break(Break::Cond),
        IrToken::Phone(Phone::Symbol("c")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("d")),
    ], lines: ONE }));
}

#[test]
fn three_conds() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: vec![Pattern::new_phone(Phone::Symbol("b"))],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            vec![
                CondPattern::new(
                    CondType::Pattern,
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("c"))]),
                    PatternList::default(),
                ),
                CondPattern::new(
                    CondType::Pattern,
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("d"))]),
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("e"))]),
                ),
                CondPattern::new(
                    CondType::Pattern,
                    PatternList::default(),
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("f"))]),
                ),
            ],
            Vec::new(),
        ).expect("pattern construction should be valid")),
    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::Break(Break::Cond),
        IrToken::Phone(Phone::Symbol("c")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Break(Break::Cond),
        IrToken::Phone(Phone::Symbol("d")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("e")),
        IrToken::Break(Break::Cond),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("f")),
    ], lines: ONE }));
}

#[test]
fn single_anti_cond() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: vec![Pattern::new_phone(Phone::Symbol("b"))],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            Vec::new(),
            vec![CondPattern::new(
                CondType::Pattern,
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("c"))]),
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("d"))]),
            )]
        ).expect("pattern construction should be valid")),
    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone(Phone::Symbol("c")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("d")),
    ], lines: ONE }));
}

#[test]
fn three_anti_conds() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: vec![Pattern::new_phone(Phone::Symbol("b"))],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            Vec::new(),
            vec![
                CondPattern::new(
                    CondType::Pattern,
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("c"))]),
                    PatternList::default(),
                ),
                CondPattern::new(
                    CondType::Pattern,
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("d"))]),
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("e"))]),
                ),
                CondPattern::new(
                    CondType::Pattern,
                    PatternList::default(),
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("f"))]),
                ),
            ]
        ).expect("pattern construction should be valid")),
    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone(Phone::Symbol("c")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone(Phone::Symbol("d")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("e")),
        IrToken::Break(Break::AntiCond),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("f")),
    ], lines: ONE }));
}

#[test]
fn cond_and_anti_cond() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: vec![Pattern::new_phone(Phone::Symbol("b"))],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            vec![CondPattern::new (
                CondType::Pattern,
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("c"))]),
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("d"))]),
            )],
            vec![CondPattern::new (
                CondType::Pattern,
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("e"))]),
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("f"))]),
            )],
        ).expect("pattern construction should be valid")),
    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::Break(Break::Cond),
        IrToken::Phone(Phone::Symbol("c")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("d")),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone(Phone::Symbol("e")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("f")),
    ], lines: ONE }));
}

#[test]
fn three_conds_and_anti_conds() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: vec![Pattern::new_phone(Phone::Symbol("b"))],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            vec![
                CondPattern::new(
                    CondType::Pattern,
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("c"))]),
                    PatternList::default(),
                ),
                CondPattern::new(
                    CondType::Pattern,
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("d"))]),
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("e"))]),
                ),
                CondPattern::new(
                    CondType::Pattern,
                    PatternList::default(),
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("f"))]),
                ),
            ],
            vec![
                CondPattern::new(
                    CondType::Pattern,
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("g"))]),
                    PatternList::default(),
                ),
                CondPattern::new(
                    CondType::Pattern,
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("h"))]),
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("i"))]),
                ),
                CondPattern::new(
                    CondType::Pattern,
                    PatternList::default(),
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("j"))]),
                ),
            ],
        ).expect("pattern construction should be valid")),
    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::Break(Break::Cond),
        IrToken::Phone(Phone::Symbol("c")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Break(Break::Cond),
        IrToken::Phone(Phone::Symbol("d")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("e")),
        IrToken::Break(Break::Cond),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("f")),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone(Phone::Symbol("g")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone(Phone::Symbol("h")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("i")),
        IrToken::Break(Break::AntiCond),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("j")),
    ], lines: ONE }));
}

// todo: test exclusives, exclusive failures (both in tokenization and application)

#[test]
fn shift_cond_repetition_input() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: Vec::new(),
        pattern: RefCell::new(
            RulePattern::new(
                PatternList::default(),
                vec![CondPattern::new(
                    CondType::Pattern,
                    PatternList::new(vec![Pattern::new_repetition(None, PatternList::new(vec![Pattern::new_any(None)]), None)]),
                    PatternList::default(),
                )],
                Vec::new(),
            ).expect("pattern construction should be valid")
        ),
    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::Break(Break::Shift(shift)),
        IrToken::Break(Break::Cond),
        IrToken::ScopeStart(ScopeType::Repetition),
        IrToken::Any,
        IrToken::ScopeEnd(ScopeType::Repetition),
        IrToken::CondType(CondType::Pattern),
    ], lines: ONE }));
}

#[test]
fn shift_anti_cond_repetition_input() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: Vec::new(),
        pattern: RefCell::new(
            RulePattern::new(
                PatternList::default(),
                Vec::new(),
                vec![CondPattern::new(
                    CondType::Pattern,
                    PatternList::new(vec![Pattern::new_repetition(None, PatternList::new(vec![Pattern::new_any(None)]), None)]),
                    PatternList::default(),
                )],
            ).expect("pattern construction should be valid")
        ),
    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::Break(Break::Shift(shift)),
        IrToken::Break(Break::AntiCond),
        IrToken::ScopeStart(ScopeType::Repetition),
        IrToken::Any,
        IrToken::ScopeEnd(ScopeType::Repetition),
        IrToken::CondType(CondType::Pattern),
    ], lines: ONE }));
}

#[test]
fn shift_cond_label_repetition_input() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: Vec::new(),
        pattern: RefCell::new(
            RulePattern::new(
                PatternList::default(),
                vec![CondPattern::new(
                    CondType::Pattern,
                    PatternList::new(vec![Pattern::new_repetition(Some("label"), PatternList::new(vec![Pattern::new_any(None)]), None)]),
                    PatternList::default(),
                )],
                Vec::new(),
            ).expect("pattern construction should be valid")
        ),
    }, lines: ONE }), build_rule(IrLine::Ir { tokens: vec![
        IrToken::Break(Break::Shift(shift)),
        IrToken::Break(Break::Cond),
        IrToken::Label("label"),
        IrToken::ScopeStart(ScopeType::Repetition),
        IrToken::Any,
        IrToken::ScopeEnd(ScopeType::Repetition),
        IrToken::CondType(CondType::Pattern),
    ], lines: ONE }));
}

#[test]
fn any_to_any() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};
    
    let any = vec![Pattern::new_any(Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }))];

    assert_eq!(
        Ok(RuleLine::Rule { rule: SoundChangeRule {
            kind: shift,
            output: any.clone(),
            pattern: RefCell::new(
                RulePattern::new(
                    PatternList::new(any),
                    Vec::new(),
                    Vec::new(),
                ).expect("pattern construction should be valid")
            ),
        }, lines: ONE }),
        build_rule(IrLine::Ir { tokens: vec![
            IrToken::Any,
            IrToken::Break(Break::Shift(shift)),
            IrToken::Any,
        ], lines: ONE })
    )
}

#[test]
fn any_any_to_any_any() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};
    
    let anys = vec![
        Pattern::new_any(Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None })),
        Pattern::new_any(Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Any, parent: None })),
    ];

    assert_eq!(
        Ok(RuleLine::Rule { rule: SoundChangeRule {
            kind: shift,
            output: anys.clone(),
            pattern: RefCell::new(
                RulePattern::new(
                    PatternList::new(anys),
                    Vec::new(),
                    Vec::new(),
                ).expect("pattern construction should be valid")
            ),
        }, lines: ONE }),
        build_rule(IrLine::Ir { tokens: vec![
            IrToken::Any,
            IrToken::Any,
            IrToken::Break(Break::Shift(shift)),
            IrToken::Any,
            IrToken::Any,
        ], lines: ONE })
    )
}

#[test]
fn labeled_any_to_any() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    let any = vec![Pattern::new_any(Some(ScopeId::Name("label")))];

    assert_eq!(
        Ok(RuleLine::Rule { rule: SoundChangeRule {
            kind: shift,
            output: any.clone(),
            pattern: RefCell::new(
                RulePattern::new(
                    PatternList::new(any),
                    Vec::new(),
                    Vec::new(),
                ).expect("pattern construction should be valid")
            ),
        }, lines: ONE }),
        build_rule(IrLine::Ir { tokens: vec![
            IrToken::Label("label"),
            IrToken::Any,
            IrToken::Break(Break::Shift(shift)),
            IrToken::Label("label"),
            IrToken::Any,
        ], lines: ONE })
    )
}

#[test]
fn selections_around_any_to_any() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    let selection = |n, sym| Pattern::new_selection(vec![vec![Pattern::new_phone(Phone::Symbol(sym))]], Some(ScopeId::IOUnlabeled { id_num: n, label_type: LabelType::Scope(ScopeType::Selection), parent: None }));
    let any = Pattern::new_any(Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }));

    assert_eq!(
        Ok(RuleLine::Rule { rule: SoundChangeRule {
            kind: shift,
            output: vec![
                    selection(0, "c"),
                    any.clone(),
                    selection(1, "d"),
                ],
            pattern: RefCell::new(
                RulePattern::new(
                    PatternList::new(vec![
                        selection(0, "a"),
                        any,
                        selection(1, "b"),
                    ]),
                    Vec::new(),
                    Vec::new(),
                ).expect("pattern construction should be valid")
            ),
        }, lines: ONE }),
        build_rule(IrLine::Ir { tokens: vec![
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::ScopeEnd(ScopeType::Selection),
            IrToken::Any,
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::ScopeEnd(ScopeType::Selection),
            IrToken::Break(Break::Shift(shift)),
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone(Phone::Symbol("c")),
            IrToken::ScopeEnd(ScopeType::Selection),
            IrToken::Any,
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone(Phone::Symbol("d")),
            IrToken::ScopeEnd(ScopeType::Selection),
        ], lines: ONE })
    );
}

#[test]
fn cond_with_scope() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule { rule: SoundChangeRule {
            kind: shift,
            output: vec![Pattern::new_phone(Phone::Symbol("b"))],
            pattern: RefCell::new(RulePattern::new(
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
                vec![CondPattern::new (
                    CondType::Pattern,
                    PatternList::new(vec![Pattern::new_selection(
                        vec![
                            vec![Pattern::new_phone(Phone::Symbol("c"))],
                            vec![Pattern::new_phone(Phone::Symbol("d"))],
                            vec![Pattern::new_phone(Phone::Symbol("e"))],
                        ],
                        None
                    )]),
                    PatternList::default(),
                )],
                Vec::new(),
            ).expect("pattern construction should be valid"))
        }, lines: ONE }),
        build_rule(IrLine::Ir { tokens: vec![
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::Break(Break::Cond),
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone(Phone::Symbol("c")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("d")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("e")),
            IrToken::ScopeEnd(ScopeType::Selection),
            IrToken::CondType(CondType::Pattern),
        ], lines: ONE })
    );
}

#[test]
fn anti_cond_with_scope() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule { rule: SoundChangeRule {
            kind: shift,
            output: vec![Pattern::new_phone(Phone::Symbol("b"))],
            pattern: RefCell::new(RulePattern::new(
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
                Vec::new(),
                vec![CondPattern::new (
                    CondType::Pattern,
                    PatternList::new(vec![Pattern::new_optional(vec![Pattern::new_phone(Phone::Symbol("c"))], Some(ScopeId::Name("label")))]),
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("d"))]),
                )],
            ).expect("pattern construction should be valid")),
        }, lines: ONE }),
        build_rule(IrLine::Ir { tokens: vec![
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::Break(Break::AntiCond),
            IrToken::Label("label"),
            IrToken::ScopeStart(ScopeType::Optional),
            IrToken::Phone(Phone::Symbol("c")),
            IrToken::ScopeEnd(ScopeType::Optional),
            IrToken::CondType(CondType::Pattern),
            IrToken::Phone(Phone::Symbol("d")),
        ], lines: ONE })
    );
}

#[test]
fn equality_cond() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule { rule: SoundChangeRule {
            kind: shift,
            output: vec![Pattern::new_phone(Phone::Symbol("b"))],
            pattern: RefCell::new(RulePattern::new(
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
                vec![CondPattern::new (
                    CondType::Match,
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("c"))]),
                    PatternList::new(vec![Pattern::new_phone(Phone::Symbol("d"))]),
                )],
                Vec::new(),
            ).expect("pattern construction should be valid"))
        }, lines: ONE }),
        build_rule(IrLine::Ir { tokens: vec![
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::Break(Break::Cond),
            IrToken::Phone(Phone::Symbol("c")),
            IrToken::CondType(CondType::Match),
            IrToken::Phone(Phone::Symbol("d")),
        ], lines: ONE })
    );
}

#[test]
fn and_cond() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    let mut cond = CondPattern::new(
        CondType::Pattern,
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("b"))]),
        PatternList::default(),
    );

    cond.add_and(AndType::And, CondPattern::new(
        CondType::Pattern,
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("c"))]),
        PatternList::default(),
    ));

    assert_eq!(
        Ok(RuleLine::Rule { rule: SoundChangeRule {
            kind: shift,
            output: Vec::new(),
            pattern: RefCell::new(RulePattern::new(
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
                vec![cond],
                Vec::new(),
            ).expect("pattern construction should be valid"))
        }, lines: ONE }),
        build_rule(IrLine::Ir { tokens: vec![
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Break(Break::Cond),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::CondType(CondType::Pattern),
            IrToken::Break(Break::And(AndType::And)),
            IrToken::Phone(Phone::Symbol("c")),
            IrToken::CondType(CondType::Pattern),
        ], lines: ONE })
    )
}

#[test]
fn and_anticond() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    let mut cond = CondPattern::new(
        CondType::Pattern,
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("b"))]),
        PatternList::default(),
    );

    cond.add_and(AndType::And, CondPattern::new(
        CondType::Pattern,
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("c"))]),
        PatternList::default(),
    ));

    assert_eq!(
        Ok(RuleLine::Rule { rule: SoundChangeRule {
            kind: shift,
            output: Vec::new(),
            pattern: RefCell::new(RulePattern::new(
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
                Vec::new(),
                vec![cond],
            ).expect("pattern construction should be valid"))
        }, lines: ONE }),
        build_rule(IrLine::Ir { tokens: vec![
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Break(Break::AntiCond),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::CondType(CondType::Pattern),
            IrToken::Break(Break::And(AndType::And)),
            IrToken::Phone(Phone::Symbol("c")),
            IrToken::CondType(CondType::Pattern),
        ], lines: ONE })
    )
}

#[test]
fn double_and() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    let mut cond = CondPattern::new(
        CondType::Pattern,
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("b"))]),
        PatternList::default(),
    );

    cond.add_and(AndType::And, CondPattern::new(
        CondType::Pattern,
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("c"))]),
        PatternList::default(),
    ));

    cond.add_and(AndType::And, CondPattern::new(
        CondType::Pattern,
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("d"))]),
        PatternList::default(),
    ));

    assert_eq!(
        Ok(RuleLine::Rule { rule: SoundChangeRule {
            kind: shift,
            output: Vec::new(),
            pattern: RefCell::new(RulePattern::new(
                PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
                vec![cond],
                Vec::new(),
            ).expect("pattern construction should be valid"))
        }, lines: ONE }),
        build_rule(IrLine::Ir { tokens: vec![
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Break(Break::Cond),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::CondType(CondType::Pattern),
            IrToken::Break(Break::And(AndType::And)),
            IrToken::Phone(Phone::Symbol("c")),
            IrToken::CondType(CondType::Pattern),
            IrToken::Break(Break::And(AndType::And)),
            IrToken::Phone(Phone::Symbol("d")),
            IrToken::CondType(CondType::Pattern),
        ], lines: ONE })
    )
}

#[test]
fn selection_sequence() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    let outer_scope_1 = ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Scope(ScopeType::Selection), parent: None };
    let outer_scope_2 = ScopeId::Name("label");
    let outer_scope_3 = ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Scope(ScopeType::Selection), parent: None };

    let expected = RuleLine::Rule { rule: SoundChangeRule {
        kind: shift,
        output: Vec::new(),
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![
                Pattern::new_selection(vec![
                    vec![Pattern::new_selection(vec![vec![Pattern::new_phone(Phone::Symbol("a"))]], Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Scope(ScopeType::Selection), parent: Some(Rc::new(outer_scope_1.clone())) }))],
                    vec![Pattern::new_selection(vec![vec![Pattern::new_phone(Phone::Symbol("b"))]], Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Scope(ScopeType::Selection), parent: Some(Rc::new(outer_scope_1.clone())) }))],
                ], Some(outer_scope_1)),

                Pattern::new_selection(vec![
                    vec![Pattern::new_selection(vec![vec![Pattern::new_phone(Phone::Symbol("c"))]], Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Scope(ScopeType::Selection), parent: Some(Rc::new(outer_scope_2.clone())) }))],
                    vec![Pattern::new_selection(vec![vec![Pattern::new_phone(Phone::Symbol("d"))]], Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Scope(ScopeType::Selection), parent: Some(Rc::new(outer_scope_2.clone())) }))],
                ], Some(outer_scope_2)),

                Pattern::new_selection(vec![
                    vec![Pattern::new_selection(vec![vec![Pattern::new_phone(Phone::Symbol("e"))]], Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Scope(ScopeType::Selection), parent: Some(Rc::new(outer_scope_3.clone())) }))],
                    vec![Pattern::new_selection(vec![vec![Pattern::new_phone(Phone::Symbol("f"))]], Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Scope(ScopeType::Selection), parent: Some(Rc::new(outer_scope_3.clone())) }))],
                ], Some(outer_scope_3)),
            ]),
            vec![CondPattern::default()],
            Vec::new(),
        ).expect("pattern construction should be valid"))
    }, lines: ONE };

    let actual = build_rule(IrLine::Ir { tokens: vec![
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::ScopeEnd(ScopeType::Selection),
        IrToken::ArgSep,
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::ScopeEnd(ScopeType::Selection),
        IrToken::ScopeEnd(ScopeType::Selection),

        IrToken::Label("label"),
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone(Phone::Symbol("c")),
        IrToken::ScopeEnd(ScopeType::Selection),
        IrToken::ArgSep,
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone(Phone::Symbol("d")),
        IrToken::ScopeEnd(ScopeType::Selection),
        IrToken::ScopeEnd(ScopeType::Selection),

        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone(Phone::Symbol("e")),
        IrToken::ScopeEnd(ScopeType::Selection),
        IrToken::ArgSep,
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone(Phone::Symbol("f")),
        IrToken::ScopeEnd(ScopeType::Selection),
        IrToken::ScopeEnd(ScopeType::Selection),

        IrToken::Break(Break::Shift(shift)),
    ], lines: ONE });

    assert_eq!(
        Ok(expected),
        actual
    )
}