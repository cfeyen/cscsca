use crate::meta_tokens::{Direction, Shift, ShiftType};
use super::*;

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
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::new("a"))],
        output: vec![RuleToken::Phone(Phone::new("b"))],
        conds: vec![Cond::default()],
        anti_conds: Vec::new()

    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone("a"),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone("b"),
    ])));
}

#[test]
fn three_to_three() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::new("a")), RuleToken::Phone(Phone::new("b")), RuleToken::Phone(Phone::new("c"))],
        output: vec![RuleToken::Phone(Phone::new("d")), RuleToken::Phone(Phone::new("e")), RuleToken::Phone(Phone::new("f"))],
        conds: vec![Cond::default()],
        anti_conds: Vec::new()

    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone("a"),
        IrToken::Phone("b"),
        IrToken::Phone("c"),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone("d"),
        IrToken::Phone("e"),
        IrToken::Phone("f"),
    ])));
}

#[test]
fn selected_three_to_selected_three() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) }), options: vec![
            vec![RuleToken::Phone(Phone::new("a"))],
            vec![RuleToken::Phone(Phone::new("b"))],
            vec![RuleToken::Phone(Phone::new("c"))],
        ] }],
        output: vec![RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) }), options: vec![
            vec![RuleToken::Phone(Phone::new("d"))],
            vec![RuleToken::Phone(Phone::new("e"))],
            vec![RuleToken::Phone(Phone::new("f"))],
        ] }],
        conds: vec![Cond::default()],
        anti_conds: Vec::new()

    })), build_rule(&IrLine::Ir(vec![
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone("a"),
        IrToken::ArgSep,
        IrToken::Phone("b"),
        IrToken::ArgSep,
        IrToken::Phone("c"),
        IrToken::ScopeEnd(ScopeType::Selection),
        IrToken::Break(Break::Shift(shift)),
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone("d"),
        IrToken::ArgSep,
        IrToken::Phone("e"),
        IrToken::ArgSep,
        IrToken::Phone("f"),
        IrToken::ScopeEnd(ScopeType::Selection),
    ])));
}

#[test]
fn labeled_selected_three_to_selected_three() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
            vec![RuleToken::Phone(Phone::new("a"))],
            vec![RuleToken::Phone(Phone::new("b"))],
            vec![RuleToken::Phone(Phone::new("c"))],
        ] }],
        output: vec![RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
            vec![RuleToken::Phone(Phone::new("d"))],
            vec![RuleToken::Phone(Phone::new("e"))],
            vec![RuleToken::Phone(Phone::new("f"))],
        ] }],
        conds: vec![Cond::default()],
        anti_conds: Vec::new()

    })), build_rule(&IrLine::Ir(vec![
        IrToken::Label("label"),
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone("a"),
        IrToken::ArgSep,
        IrToken::Phone("b"),
        IrToken::ArgSep,
        IrToken::Phone("c"),
        IrToken::ScopeEnd(ScopeType::Selection),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Label("label"),
        IrToken::ScopeStart(ScopeType::Selection),
        IrToken::Phone("d"),
        IrToken::ArgSep,
        IrToken::Phone("e"),
        IrToken::ArgSep,
        IrToken::Phone("f"),
        IrToken::ScopeEnd(ScopeType::Selection),
    ])));
}

#[test]
fn labeled_phone_to_phone() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(
        Err((RuleStructureError::LabelNotFollowedByScope("label"), 1)),
        build_rules(&[IrLine::Ir(vec![
            IrToken::Label("label"),
            IrToken::Phone("a"),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Phone("b"),
        ])])
    );
}

#[test]
fn no_shift() {
    assert_eq!(
        Err((RuleStructureError::NoShift, 1)),
        build_rules(&[IrLine::Ir(vec![
            IrToken::Phone("a"),
            IrToken::Phone("b"),
            IrToken::Phone("c"),
        ])])
    );
}

#[test]
fn no_left() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::new("a"))],
        output: Vec::new(),
        conds: vec![Cond::default()],
        anti_conds: Vec::new()
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone("a"),
        IrToken::Break(Break::Shift(shift)),
    ])));
}

#[test]
fn single_option() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::OptionalScope { id: Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Optional) }), content: vec![
            RuleToken::Phone(Phone::new("a")),
        ] }],
        output: Vec::new(),
        conds: vec![Cond::default()],
        anti_conds: Vec::new()
    })), build_rule(&IrLine::Ir(vec![
        IrToken::ScopeStart(ScopeType::Optional),
        IrToken::Phone("a"),
        IrToken::ScopeEnd(ScopeType::Optional),
        IrToken::Break(Break::Shift(shift)),
    ])));
}

#[test]
fn nested_scopes() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![
                RuleToken::SelectionScope {
                    id: Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) }),
                    options: vec![
                        vec![RuleToken::Phone(Phone::new("a"))],
                        vec![RuleToken::OptionalScope {
                            id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Scope(ScopeType::Optional), parent: Some(Arc::new(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) })) }),
                            content: vec![RuleToken::Phone(Phone::new("b"))],
                        }],
                        vec![RuleToken::OptionalScope {
                            id: Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Scope(ScopeType::Optional), parent: Some(Arc::new(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) })) }),
                            content: vec![RuleToken::Phone(Phone::new("c"))],
                        }],
                        vec![RuleToken::Phone(Phone::new("d"))],
                    ],
                }
            ],
            output: vec![
                RuleToken::SelectionScope {
                    id: Some(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) }),
                    options: vec![
                        vec![RuleToken::Phone(Phone::new("e"))],
                        vec![RuleToken::OptionalScope {
                            id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Scope(ScopeType::Optional), parent: Some(Arc::new(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) })) }),
                            content: vec![RuleToken::Phone(Phone::new("f"))],
                        }],
                        vec![RuleToken::OptionalScope {
                            id: Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Scope(ScopeType::Optional), parent: Some(Arc::new(ScopeId::IOUnlabeled { parent: None, id_num: 0, label_type: LabelType::Scope(ScopeType::Selection) })) }),
                            content: vec![RuleToken::Phone(Phone::new("g"))],
                        }],
                        vec![RuleToken::Phone(Phone::new("h"))],
                    ],
                }
            ],
            conds: vec![Cond::default()],
            anti_conds: Vec::new(),
        })),
        build_rule(&IrLine::Ir(vec![
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone("a"),
            IrToken::ArgSep,
            IrToken::ScopeStart(ScopeType::Optional),
            IrToken::Phone("b"),
            IrToken::ScopeEnd(ScopeType::Optional),
            IrToken::ArgSep,
            IrToken::ScopeStart(ScopeType::Optional),
            IrToken::Phone("c"),
            IrToken::ScopeEnd(ScopeType::Optional),
            IrToken::ArgSep,
            IrToken::Phone("d"),
            IrToken::ScopeEnd(ScopeType::Selection),
            IrToken::Break(Break::Shift(shift)),
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone("e"),
            IrToken::ArgSep,
            IrToken::ScopeStart(ScopeType::Optional),
            IrToken::Phone("f"),
            IrToken::ScopeEnd(ScopeType::Optional),
            IrToken::ArgSep,
            IrToken::ScopeStart(ScopeType::Optional),
            IrToken::Phone("g"),
            IrToken::ScopeEnd(ScopeType::Optional),
            IrToken::ArgSep,
            IrToken::Phone("h"),
            IrToken::ScopeEnd(ScopeType::Selection),
        ]))
    );
}

#[test]
fn single_cond() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::new("a"))],
        output: vec![RuleToken::Phone(Phone::new("b"))],
        conds: vec![Cond::new(
            CondType::MatchInput,
            vec![RuleToken::Phone(Phone::new("c"))],
            vec![RuleToken::Phone(Phone::new("d"))],
        )],
        anti_conds: Vec::new()
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone("a"),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone("b"),
        IrToken::Break(Break::Cond),
        IrToken::Phone("c"),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Phone("d"),
    ])));
}

#[test]
fn three_conds() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::new("a"))],
        output: vec![RuleToken::Phone(Phone::new("b"))],
        conds: vec![
            Cond::new(
                CondType::MatchInput,
                vec![RuleToken::Phone(Phone::new("c"))],
                Vec::new(),
            ),
            Cond::new(
                CondType::MatchInput,
                vec![RuleToken::Phone(Phone::new("d"))],
                vec![RuleToken::Phone(Phone::new("e"))],
            ),
            Cond::new(
                CondType::MatchInput,
                Vec::new(),
                vec![RuleToken::Phone(Phone::new("f"))],
            ),
        ],
        anti_conds: Vec::new()
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone("a"),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone("b"),
        IrToken::Break(Break::Cond),
        IrToken::Phone("c"),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Break(Break::Cond),
        IrToken::Phone("d"),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Phone("e"),
        IrToken::Break(Break::Cond),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Phone("f"),
    ])));
}

#[test]
fn single_anti_cond() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::new("a"))],
        output: vec![RuleToken::Phone(Phone::new("b"))],
        conds: vec![Cond::default()],
        anti_conds: vec![Cond::new(
            CondType::MatchInput,
            vec![RuleToken::Phone(Phone::new("c"))],
            vec![RuleToken::Phone(Phone::new("d"))],
        )],
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone("a"),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone("b"),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone("c"),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Phone("d"),
    ])));
}

#[test]
fn three_anti_conds() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::new("a"))],
        output: vec![RuleToken::Phone(Phone::new("b"))],
        conds: vec![Cond::default()],
        anti_conds: vec![
            Cond::new(
                CondType::MatchInput,
                vec![RuleToken::Phone(Phone::new("c"))],
                Vec::new(),
            ),
            Cond::new(
                CondType::MatchInput,
                vec![RuleToken::Phone(Phone::new("d"))],
                vec![RuleToken::Phone(Phone::new("e"))],
            ),
            Cond::new(
                CondType::MatchInput,
                Vec::new(),
                vec![RuleToken::Phone(Phone::new("f"))],
            ),
        ],
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone("a"),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone("b"),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone("c"),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone("d"),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Phone("e"),
        IrToken::Break(Break::AntiCond),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Phone("f"),
    ])));
}

#[test]
fn cond_and_anti_cond() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::new("a"))],
        output: vec![RuleToken::Phone(Phone::new("b"))],
        conds: vec![Cond::new (
            CondType::MatchInput,
            vec![RuleToken::Phone(Phone::new("c"))],
            vec![RuleToken::Phone(Phone::new("d"))],
        )],
        anti_conds: vec![Cond::new (
            CondType::MatchInput,
            vec![RuleToken::Phone(Phone::new("e"))],
            vec![RuleToken::Phone(Phone::new("f"))],
        )],
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone("a"),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone("b"),
        IrToken::Break(Break::Cond),
        IrToken::Phone("c"),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Phone("d"),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone("e"),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Phone("f"),
    ])));
}

#[test]
fn three_conds_and_anti_conds() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: vec![RuleToken::Phone(Phone::new("a"))],
        output: vec![RuleToken::Phone(Phone::new("b"))],
        conds: vec![
            Cond::new(
                CondType::MatchInput,
                vec![RuleToken::Phone(Phone::new("c"))],
                Vec::new(),
            ),
            Cond::new(
                CondType::MatchInput,
                vec![RuleToken::Phone(Phone::new("d"))],
                vec![RuleToken::Phone(Phone::new("e"))],
            ),
            Cond::new(
                CondType::MatchInput,
                Vec::new(),
                vec![RuleToken::Phone(Phone::new("f"))],
            ),
        ],
        anti_conds: vec![
            Cond::new(
                CondType::MatchInput,
                vec![RuleToken::Phone(Phone::new("g"))],
                Vec::new(),
            ),
            Cond::new(
                CondType::MatchInput,
                vec![RuleToken::Phone(Phone::new("h"))],
                vec![RuleToken::Phone(Phone::new("i"))],
            ),
            Cond::new(
                CondType::MatchInput,
                Vec::new(),
                vec![RuleToken::Phone(Phone::new("j"))],
            ),
        ],
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Phone("a"),
        IrToken::Break(Break::Shift(shift)),
        IrToken::Phone("b"),
        IrToken::Break(Break::Cond),
        IrToken::Phone("c"),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Break(Break::Cond),
        IrToken::Phone("d"),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Phone("e"),
        IrToken::Break(Break::Cond),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Phone("f"),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone("g"),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone("h"),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Phone("i"),
        IrToken::Break(Break::AntiCond),
        IrToken::CondFocus(CondType::MatchInput),
        IrToken::Phone("j"),
    ])));
}

#[test]
fn shift_cond_gap_input() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: Vec::new(),
        output: Vec::new(),
        conds: vec![Cond::new(
            CondType::MatchInput,
            vec![RuleToken::Gap { id: None }],
            Vec::new(),
        )],
        anti_conds: Vec::new(),
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Break(Break::Shift(shift)),
        IrToken::Break(Break::Cond),
        IrToken::Gap,
        IrToken::CondFocus(CondType::MatchInput),
    ])));
}

#[test]
fn shift_anti_cond_gap_input() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: Vec::new(),
        output: Vec::new(),
        conds: vec![Cond::default()],
        anti_conds: vec![Cond::new(
            CondType::MatchInput,
            vec![RuleToken::Gap { id: None }],
            Vec::new(),
    )],
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Break(Break::Shift(shift)),
        IrToken::Break(Break::AntiCond),
        IrToken::Gap,
        IrToken::CondFocus(CondType::MatchInput),
    ])));
}

#[test]
fn shift_cond_label_gap_input() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(Ok(RuleLine::Rule(SoundChangeRule {
        kind: shift,
        input: Vec::new(),
        output: Vec::new(),
        conds: vec![Cond::new(
            CondType::MatchInput,
            vec![RuleToken::Gap { id: Some("label") }],
            Vec::new(),
    )],
        anti_conds: Vec::new(),
    })), build_rule(&IrLine::Ir(vec![
        IrToken::Break(Break::Shift(shift)),
        IrToken::Break(Break::Cond),
        IrToken::Label("label"),
        IrToken::Gap,
        IrToken::CondFocus(CondType::MatchInput),
    ])));
}

#[test]
fn any_to_any() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

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
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

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
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

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
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![
                    RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Scope(ScopeType::Selection), parent: None }), options: vec![vec![RuleToken::Phone(Phone::new("a"))]] },
                    RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) },
                    RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Scope(ScopeType::Selection), parent: None }), options: vec![vec![RuleToken::Phone(Phone::new("b"))]] },
                ],
            output: vec![
                    RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Scope(ScopeType::Selection), parent: None }), options: vec![vec![RuleToken::Phone(Phone::new("c"))]] },
                    RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) },
                    RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled { id_num: 1, label_type: LabelType::Scope(ScopeType::Selection), parent: None }), options: vec![vec![RuleToken::Phone(Phone::new("d"))]] },
                ],
            conds: vec![Cond::default()],
            anti_conds: Vec::new(),
        })),
        build_rule(&IrLine::Ir(vec![
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone("a"),
            IrToken::ScopeEnd(ScopeType::Selection),
            IrToken::Any,
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone("b"),
            IrToken::ScopeEnd(ScopeType::Selection),
            IrToken::Break(Break::Shift(shift)),
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone("c"),
            IrToken::ScopeEnd(ScopeType::Selection),
            IrToken::Any,
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone("d"),
            IrToken::ScopeEnd(ScopeType::Selection),
        ]))
    );
}

#[test]
fn cond_with_scope() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![RuleToken::Phone(Phone::new("a"))],
            output: vec![RuleToken::Phone(Phone::new("b"))],
            conds: vec![Cond::new (
                    CondType::MatchInput,
                    vec![RuleToken::SelectionScope { id: None, options: vec![
                            vec![RuleToken::Phone(Phone::new("c"))],
                            vec![RuleToken::Phone(Phone::new("d"))],
                            vec![RuleToken::Phone(Phone::new("e"))],
                        ]}],
                    Vec::new(),
                )],
            anti_conds: Vec::new(),
        })),
        build_rule(&IrLine::Ir(vec![
            IrToken::Phone("a"),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Phone("b"),
            IrToken::Break(Break::Cond),
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone("c"),
            IrToken::ArgSep,
            IrToken::Phone("d"),
            IrToken::ArgSep,
            IrToken::Phone("e"),
            IrToken::ScopeEnd(ScopeType::Selection),
            IrToken::CondFocus(CondType::MatchInput),
        ]))
    );
}

#[test]
fn anti_cond_with_scope() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![RuleToken::Phone(Phone::new("a"))],
            output: vec![RuleToken::Phone(Phone::new("b"))],
            conds: vec![Cond::default()],
            anti_conds: vec![Cond::new (
                CondType::MatchInput,
                vec![RuleToken::OptionalScope { id: Some(ScopeId::Name("label")), content: vec![
                        RuleToken::Phone(Phone::new("c"))
                    ]}],
                vec![RuleToken::Phone(Phone::new("d"))],
            )],
        })),
        build_rule(&IrLine::Ir(vec![
            IrToken::Phone("a"),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Phone("b"),
            IrToken::Break(Break::AntiCond),
            IrToken::Label("label"),
            IrToken::ScopeStart(ScopeType::Optional),
            IrToken::Phone("c"),
            IrToken::ScopeEnd(ScopeType::Optional),
            IrToken::CondFocus(CondType::MatchInput),
            IrToken::Phone("d"),
        ]))
    );
}

#[test]
fn equality_cond() {
    let shift = Shift { dir: Direction::LTR, kind: ShiftType::Move};

    assert_eq!(
        Ok(RuleLine::Rule(SoundChangeRule {
            kind: shift,
            input: vec![RuleToken::Phone(Phone::new("a"))],
            output: vec![RuleToken::Phone(Phone::new("b"))],
            conds: vec![Cond::new (
                CondType::Equality,
                vec![RuleToken::Phone(Phone::new("c"))],
                vec![RuleToken::Phone(Phone::new("d"))],
            )],
            anti_conds: Vec::new(),
        })),
        build_rule(&IrLine::Ir(vec![
            IrToken::Phone("a"),
            IrToken::Break(Break::Shift(shift)),
            IrToken::Phone("b"),
            IrToken::Break(Break::Cond),
            IrToken::Phone("c"),
            IrToken::CondFocus(CondType::Equality),
            IrToken::Phone("d"),
        ]))
    );
}