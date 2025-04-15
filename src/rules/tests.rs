use crate::{meta_tokens::{Direction, Shift, ShiftType}, phones::Phone};
use super::*;

/// Builds a sound change rules out of lines of ir tokens,
/// if there is an error it is returned with its line number
/// 
/// Note: the ir tokens should be checked for proper structure before being passed to this function
#[cfg(test)]
fn build_rules<'s>(token_lines: &'s [IrLine<'s>]) -> Result<Vec<RuleLine<'s>>, (RuleStructureError<'s>, usize)> {
    let token_lines = token_lines
        .iter()
        .enumerate()
        .map(|(num, line)| (num + 1, line));

    let mut rules = Vec::new();

    for (line_num, line) in token_lines {
        match build_rule(line) {
            Ok(rule) => rules.push(rule),
            Err(e) => return Err((e, line_num))
        }
    }

    Ok(rules)
}

#[test]
fn no_rule() {
    assert_eq!(Ok(Vec::new()), build_rules(&Vec::new()));
}

#[test]
fn empty_line() {
    assert_eq!(Ok(RuleLine::Empty), build_rule(&IrLine::Empty));
    assert_eq!(Ok(RuleLine::Empty), build_rule(&IrLine::Ir(Vec::new())));
}

#[test]
fn one_to_one() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: vec![RuleToken::Phone(Phone::Symbol("b"))],
        conds: vec![Cond::default()],
        anti_conds: Vec::new()

    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone(Phone::Symbol("b")),
    ])));
}

#[test]
fn three_to_three() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::Symbol("a")), RuleToken::Phone(Phone::Symbol("b")), RuleToken::Phone(Phone::Symbol("c"))],
        output: vec![RuleToken::Phone(Phone::Symbol("d")), RuleToken::Phone(Phone::Symbol("e")), RuleToken::Phone(Phone::Symbol("f"))],
        conds: vec![Cond::default()],
        anti_conds: Vec::new()

    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::Phone(Phone::Symbol("c")),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone(Phone::Symbol("d")),
        IrToken::Phone(Phone::Symbol("e")),
        IrToken::Phone(Phone::Symbol("f")),
    ])));
}

#[test]
fn selected_three_to_selected_three() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) }), options: vec![
            vec![RuleToken::Phone(Phone::Symbol("a"))],
            vec![RuleToken::Phone(Phone::Symbol("b"))],
            vec![RuleToken::Phone(Phone::Symbol("c"))],
        ] }],
        output: vec![RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) }), options: vec![
            vec![RuleToken::Phone(Phone::Symbol("d"))],
            vec![RuleToken::Phone(Phone::Symbol("e"))],
            vec![RuleToken::Phone(Phone::Symbol("f"))],
        ] }],
        conds: vec![Cond::default()],
        anti_conds: Vec::new()

    })), build_rule(&IrLine::Ir(vec![
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
    ])));
}

#[test]
fn labeled_selected_three_to_selected_three() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
            vec![RuleToken::Phone(Phone::Symbol("a"))],
            vec![RuleToken::Phone(Phone::Symbol("b"))],
            vec![RuleToken::Phone(Phone::Symbol("c"))],
        ] }],
        output: vec![RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
            vec![RuleToken::Phone(Phone::Symbol("d"))],
            vec![RuleToken::Phone(Phone::Symbol("e"))],
            vec![RuleToken::Phone(Phone::Symbol("f"))],
        ] }],
        conds: vec![Cond::default()],
        anti_conds: Vec::new()

    })), build_rule(&IrLine::Ir(vec![
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
    ])));
}

#[test]
fn labeled_phone_to_phone() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(
        Err((RuleStructureError::LabelNotFollowedByScope("label"), 1)),
        build_rules(&[IrLine::Ir(vec![
            IrToken::Label("label"),
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Phone(Phone::Symbol("b")),
        ])])
    );
}

#[test]
fn no_shift() {
    assert_eq!(
        Err((RuleStructureError::NoShift, 1)),
        build_rules(&[IrLine::Ir(vec![
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::Phone(Phone::Symbol("c")),
        ])])
    );
}

#[test]
fn no_left() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: Vec::new(),
        conds: vec![Cond::default()],
        anti_conds: Vec::new()
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Break(Break::Shift(shift)),
    ])));
}

#[test]
fn single_option() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::OptionalScope { id: Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Optional) }), content: vec![
            RuleToken::Phone(Phone::Symbol("a")),
        ] }],
        output: Vec::new(),
        conds: vec![Cond::default()],
        anti_conds: Vec::new()
    })), build_rule(&IrLine::Ir(vec![
        IrToken::ScopeStart(ScopeType::Optional),
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::ScopeEnd(ScopeType::Optional),
        IrToken::Break(Break::Shift(shift)),
    ])));
}

#[test]
fn nested_scopes() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![
                RuleToken::SelectionScope {
                    id: Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) }),
                    options: vec![
                        vec![RuleToken::Phone(Phone::Symbol("a"))],
                        vec![RuleToken::OptionalScope {
                            id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Scope(ScopeType::Optional), parent: Some(Arc::new(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) })) }),
                            content: vec![RuleToken::Phone(Phone::Symbol("b"))],
                        }],
                        vec![RuleToken::OptionalScope {
                            id: Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Scope(ScopeType::Optional), parent: Some(Arc::new(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) })) }),
                            content: vec![RuleToken::Phone(Phone::Symbol("c"))],
                        }],
                        vec![RuleToken::Phone(Phone::Symbol("d"))],
                    ],
                }
            ],
            output: vec![
                RuleToken::SelectionScope {
                    id: Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) }),
                    options: vec![
                        vec![RuleToken::Phone(Phone::Symbol("e"))],
                        vec![RuleToken::OptionalScope {
                            id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Scope(ScopeType::Optional), parent: Some(Arc::new(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) })) }),
                            content: vec![RuleToken::Phone(Phone::Symbol("f"))],
                        }],
                        vec![RuleToken::OptionalScope {
                            id: Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Scope(ScopeType::Optional), parent: Some(Arc::new(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) })) }),
                            content: vec![RuleToken::Phone(Phone::Symbol("g"))],
                        }],
                        vec![RuleToken::Phone(Phone::Symbol("h"))],
                    ],
                }
            ],
            conds: vec![Cond::default()],
            anti_conds: Vec::new(),
        })),
        build_rule(&IrLine::Ir(vec![
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
        ]))
    );
}

#[test]
fn single_cond() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: vec![RuleToken::Phone(Phone::Symbol("b"))],
        conds: vec![Cond::new(
            CondType::Pattern,
            vec![RuleToken::Phone(Phone::Symbol("c"))],
            vec![RuleToken::Phone(Phone::Symbol("d"))],
        )],
        anti_conds: Vec::new()
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::Break(Break::Cond),
        IrToken::Phone(Phone::Symbol("c")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("d")),
    ])));
}

#[test]
fn three_conds() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: vec![RuleToken::Phone(Phone::Symbol("b"))],
        conds: vec![
            Cond::new(
                CondType::Pattern,
                vec![RuleToken::Phone(Phone::Symbol("c"))],
                Vec::new(),
            ),
            Cond::new(
                CondType::Pattern,
                vec![RuleToken::Phone(Phone::Symbol("d"))],
                vec![RuleToken::Phone(Phone::Symbol("e"))],
            ),
            Cond::new(
                CondType::Pattern,
                Vec::new(),
                vec![RuleToken::Phone(Phone::Symbol("f"))],
            ),
        ],
        anti_conds: Vec::new()
    })), build_rule(&IrLine::Ir(vec![
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
    ])));
}

#[test]
fn single_anti_cond() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: vec![RuleToken::Phone(Phone::Symbol("b"))],
        conds: vec![Cond::default()],
        anti_conds: vec![Cond::new(
            CondType::Pattern,
            vec![RuleToken::Phone(Phone::Symbol("c"))],
            vec![RuleToken::Phone(Phone::Symbol("d"))],
        )],
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone(Phone::Symbol("c")),
        IrToken::CondType(CondType::Pattern),
        IrToken::Phone(Phone::Symbol("d")),
    ])));
}

#[test]
fn three_anti_conds() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: vec![RuleToken::Phone(Phone::Symbol("b"))],
        conds: vec![Cond::default()],
        anti_conds: vec![
            Cond::new(
                CondType::Pattern,
                vec![RuleToken::Phone(Phone::Symbol("c"))],
                Vec::new(),
            ),
            Cond::new(
                CondType::Pattern,
                vec![RuleToken::Phone(Phone::Symbol("d"))],
                vec![RuleToken::Phone(Phone::Symbol("e"))],
            ),
            Cond::new(
                CondType::Pattern,
                Vec::new(),
                vec![RuleToken::Phone(Phone::Symbol("f"))],
            ),
        ],
    })), build_rule(&IrLine::Ir(vec![
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
    ])));
}

#[test]
fn cond_and_anti_cond() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: vec![RuleToken::Phone(Phone::Symbol("b"))],
        conds: vec![Cond::new (
            CondType::Pattern,
            vec![RuleToken::Phone(Phone::Symbol("c"))],
            vec![RuleToken::Phone(Phone::Symbol("d"))],
        )],
        anti_conds: vec![Cond::new (
            CondType::Pattern,
            vec![RuleToken::Phone(Phone::Symbol("e"))],
            vec![RuleToken::Phone(Phone::Symbol("f"))],
        )],
    })), build_rule(&IrLine::Ir(vec![
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
    ])));
}

#[test]
fn three_conds_and_anti_conds() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: vec![RuleToken::Phone(Phone::Symbol("b"))],
        conds: vec![
            Cond::new(
                CondType::Pattern,
                vec![RuleToken::Phone(Phone::Symbol("c"))],
                Vec::new(),
            ),
            Cond::new(
                CondType::Pattern,
                vec![RuleToken::Phone(Phone::Symbol("d"))],
                vec![RuleToken::Phone(Phone::Symbol("e"))],
            ),
            Cond::new(
                CondType::Pattern,
                Vec::new(),
                vec![RuleToken::Phone(Phone::Symbol("f"))],
            ),
        ],
        anti_conds: vec![
            Cond::new(
                CondType::Pattern,
                vec![RuleToken::Phone(Phone::Symbol("g"))],
                Vec::new(),
            ),
            Cond::new(
                CondType::Pattern,
                vec![RuleToken::Phone(Phone::Symbol("h"))],
                vec![RuleToken::Phone(Phone::Symbol("i"))],
            ),
            Cond::new(
                CondType::Pattern,
                Vec::new(),
                vec![RuleToken::Phone(Phone::Symbol("j"))],
            ),
        ],
    })), build_rule(&IrLine::Ir(vec![
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
    ])));
}

#[test]
fn shift_cond_gap_input() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: Vec::new(),
        output: Vec::new(),
        conds: vec![Cond::new(
            CondType::Pattern,
            vec![RuleToken::Gap { id: None }],
            Vec::new(),
        )],
        anti_conds: Vec::new(),
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Break(Break::Shift(shift)),
        IrToken::Break(Break::Cond),
        IrToken::Gap,
        IrToken::CondType(CondType::Pattern),
    ])));
}

#[test]
fn shift_anti_cond_gap_input() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: Vec::new(),
        output: Vec::new(),
        conds: vec![Cond::default()],
        anti_conds: vec![Cond::new(
            CondType::Pattern,
            vec![RuleToken::Gap { id: None }],
            Vec::new(),
    )],
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Break(Break::Shift(shift)),
        IrToken::Break(Break::AntiCond),
        IrToken::Gap,
        IrToken::CondType(CondType::Pattern),
    ])));
}

#[test]
fn shift_cond_label_gap_input() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: Vec::new(),
        output: Vec::new(),
        conds: vec![Cond::new(
            CondType::Pattern,
            vec![RuleToken::Gap { id: Some("label") }],
            Vec::new(),
    )],
        anti_conds: Vec::new(),
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Break(Break::Shift(shift)),
        IrToken::Break(Break::Cond),
        IrToken::Label("label"),
        IrToken::Gap,
        IrToken::CondType(CondType::Pattern),
    ])));
}

#[test]
fn any_to_any() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) }],
            output: vec![RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) }],
            conds: vec![Cond::default()],
            anti_conds: Vec::new(),
        })),
        build_rule(&IrLine::Ir(vec![
            IrToken::Any,
            IrToken::Break(Break::Shift(shift)),
            IrToken::Any,
        ]))
    )
}

#[test]
fn any_any_to_any_any() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![
                RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) },
                RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Any, parent: None }) },
            ],
            output: vec![
                RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) },
                RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Any, parent: None }) },
            ],
            conds: vec![Cond::default()],
            anti_conds: Vec::new(),
        })),
        build_rule(&IrLine::Ir(vec![
            IrToken::Any,
            IrToken::Any,
            IrToken::Break(Break::Shift(shift)),
            IrToken::Any,
            IrToken::Any,
        ]))
    )
}

#[test]
fn labeled_any_to_any() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![RuleToken::Any { id: Some(ScopeId::Name("label")) }],
            output: vec![RuleToken::Any { id: Some(ScopeId::Name("label")) }],
            conds: vec![Cond::default()],
            anti_conds: Vec::new(),
        })),
        build_rule(&IrLine::Ir(vec![
            IrToken::Label("label"),
            IrToken::Any,
            IrToken::Break(Break::Shift(shift)),
            IrToken::Label("label"),
            IrToken::Any,
        ]))
    )
}

#[test]
fn selections_around_any_to_any() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![
                    RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Scope(ScopeType::Selection), parent: None }), options: vec![vec![RuleToken::Phone(Phone::Symbol("a"))]] },
                    RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) },
                    RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Scope(ScopeType::Selection), parent: None }), options: vec![vec![RuleToken::Phone(Phone::Symbol("b"))]] },
                ],
            output: vec![
                    RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Scope(ScopeType::Selection), parent: None }), options: vec![vec![RuleToken::Phone(Phone::Symbol("c"))]] },
                    RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) },
                    RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Scope(ScopeType::Selection), parent: None }), options: vec![vec![RuleToken::Phone(Phone::Symbol("d"))]] },
                ],
            conds: vec![Cond::default()],
            anti_conds: Vec::new(),
        })),
        build_rule(&IrLine::Ir(vec![
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
        ]))
    );
}

#[test]
fn cond_with_scope() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![RuleToken::Phone(Phone::Symbol("a"))],
            output: vec![RuleToken::Phone(Phone::Symbol("b"))],
            conds: vec![Cond::new (
                    CondType::Pattern,
                    vec![RuleToken::SelectionScope { id: None, options: vec![
                            vec![RuleToken::Phone(Phone::Symbol("c"))],
                            vec![RuleToken::Phone(Phone::Symbol("d"))],
                            vec![RuleToken::Phone(Phone::Symbol("e"))],
                        ]}],
                    Vec::new(),
                )],
            anti_conds: Vec::new(),
        })),
        build_rule(&IrLine::Ir(vec![
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
        ]))
    );
}

#[test]
fn anti_cond_with_scope() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![RuleToken::Phone(Phone::Symbol("a"))],
            output: vec![RuleToken::Phone(Phone::Symbol("b"))],
            conds: vec![Cond::default()],
            anti_conds: vec![Cond::new (
                CondType::Pattern,
                vec![RuleToken::OptionalScope { id: Some(ScopeId::Name("label")), content: vec![
                        RuleToken::Phone(Phone::Symbol("c"))
                    ]}],
                vec![RuleToken::Phone(Phone::Symbol("d"))],
            )],
        })),
        build_rule(&IrLine::Ir(vec![
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
        ]))
    );
}

#[test]
fn equality_cond() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![RuleToken::Phone(Phone::Symbol("a"))],
            output: vec![RuleToken::Phone(Phone::Symbol("b"))],
            conds: vec![Cond::new (
                CondType::Match,
                vec![RuleToken::Phone(Phone::Symbol("c"))],
                vec![RuleToken::Phone(Phone::Symbol("d"))],
            )],
            anti_conds: Vec::new(),
        })),
        build_rule(&IrLine::Ir(vec![
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::Break(Break::Cond),
            IrToken::Phone(Phone::Symbol("c")),
            IrToken::CondType(CondType::Match),
            IrToken::Phone(Phone::Symbol("d")),
        ]))
    );
}

#[test]
fn and_cond() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    let mut cond = Cond::new(
        CondType::Pattern,
        vec![RuleToken::Phone(Phone::Symbol("c"))],
        Vec::new(),
    );

    cond.set_and(Cond::new(
        CondType::Pattern,
        vec![RuleToken::Phone(Phone::Symbol("b"))],
        Vec::new(),
    ));

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![RuleToken::Phone(Phone::Symbol("a"))],
            output: Vec::new(),
            conds: vec![cond],
            anti_conds: Vec::new(),
        })),
        build_rule(&IrLine::Ir(vec![
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Break(Break::Cond),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::CondType(CondType::Pattern),
            IrToken::Break(Break::And),
            IrToken::Phone(Phone::Symbol("c")),
            IrToken::CondType(CondType::Pattern),
        ]))
    )
}

#[test]
fn and_anticond() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    let mut cond = Cond::new(
        CondType::Pattern,
        vec![RuleToken::Phone(Phone::Symbol("c"))],
        Vec::new(),
    );

    cond.set_and(Cond::new(
        CondType::Pattern,
        vec![RuleToken::Phone(Phone::Symbol("b"))],
        Vec::new(),
    ));

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![RuleToken::Phone(Phone::Symbol("a"))],
            output: Vec::new(),
            conds: vec![Cond::default()],
            anti_conds: vec![cond],
        })),
        build_rule(&IrLine::Ir(vec![
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Break(Break::AntiCond),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::CondType(CondType::Pattern),
            IrToken::Break(Break::And),
            IrToken::Phone(Phone::Symbol("c")),
            IrToken::CondType(CondType::Pattern),
        ]))
    )
}
#[test]
fn double_and() {
    let shift = Shift { dir: Direction::Ltr, kind: ShiftType::Move};

    let mut cond = Cond::new(
        CondType::Pattern,
        vec![RuleToken::Phone(Phone::Symbol("d"))],
        Vec::new(),
    );

    let mut cond_2 = Cond::new(
        CondType::Pattern,
        vec![RuleToken::Phone(Phone::Symbol("c"))],
        Vec::new(),
    );

    cond_2.set_and(Cond::new(
        CondType::Pattern,
        vec![RuleToken::Phone(Phone::Symbol("b"))],
        Vec::new(),
    ));

    cond.set_and(cond_2);

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![RuleToken::Phone(Phone::Symbol("a"))],
            output: Vec::new(),
            conds: vec![cond],
            anti_conds: Vec::new(),
        })),
        build_rule(&IrLine::Ir(vec![
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Break(Break::Cond),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::CondType(CondType::Pattern),
            IrToken::Break(Break::And),
            IrToken::Phone(Phone::Symbol("c")),
            IrToken::CondType(CondType::Pattern),
            IrToken::Break(Break::And),
            IrToken::Phone(Phone::Symbol("d")),
            IrToken::CondType(CondType::Pattern),
        ]))
    )
}