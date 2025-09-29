use std::cell::RefCell;

use crate::{executor::runtime::DEFAULT_LINE_APPLICATION_LIMIT, matcher::patterns::{cond::CondPattern, list::PatternList, rule::RulePattern}, tokens::{CondType, LabelType, ScopeId, ScopeType, Shift}};
use super::*;

#[test]
fn apply_empty_rule_to_no_phones() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        output: Vec::new(),
        pattern: RefCell::new(RulePattern::new(
            PatternList::default(),
            Vec::new(),
            Vec::new(),
        ).expect("rule structure should be valid")),
    };
    
    assert_eq!(Ok(()), apply(&rule, &mut Vec::new(), Some(DEFAULT_LINE_APPLICATION_LIMIT)));
}

// #[test]
// fn apply_empty_rule_to_one_phone() {
//     let rule = SoundChangeRule {
//         kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
//         input: Vec::new(),
//         output: Vec::new(),
//         conds: vec![CondPattern::default()],
//         anti_conds: Vec::new(),
//     };
    
//     assert_eq!(Err(ApplicationError::MatchError(MatchError::EmptyInput)), apply(&rule, &mut vec![Phone::Symbol("a")], Some(DEFAULT_LINE_APPLICATION_LIMIT)));
// }

#[test]
fn one_to_one_shift() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        output: vec![Pattern::new_phone(Phone::Symbol("b"))],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            Vec::new(),
            Vec::new(),
        ).expect("rule structure should be valid")),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("c"), Phone::Symbol("a")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, Some(DEFAULT_LINE_APPLICATION_LIMIT)));

    assert_eq!(vec![Phone::Symbol("b"), Phone::Symbol("c"), Phone::Symbol("b")], phones);
}

#[test]
fn one_to_two_shift() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        output: vec![
            Pattern::new_phone(Phone::Symbol("b")),
            Pattern::new_phone(Phone::Symbol("c")),
        ],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            Vec::new(),
            Vec::new(),
        ).expect("rule structure should be valid")),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("d"), Phone::Symbol("a")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, Some(DEFAULT_LINE_APPLICATION_LIMIT)));

    assert_eq!(vec![Phone::Symbol("b"), Phone::Symbol("c"), Phone::Symbol("d"), Phone::Symbol("b"), Phone::Symbol("c")], phones);
}

#[test]
fn two_to_one_shift() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        output: vec![Pattern::new_phone(Phone::Symbol("c"))],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![
                Pattern::new_phone(Phone::Symbol("a")),
                Pattern::new_phone(Phone::Symbol("b")),
            ]),
            Vec::new(),
            Vec::new(),
        ).expect("rule structure should be valid")),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("d"), Phone::Symbol("a"), Phone::Symbol("b")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, Some(DEFAULT_LINE_APPLICATION_LIMIT)));

    assert_eq!(vec![Phone::Symbol("c"), Phone::Symbol("d"), Phone::Symbol("c")], phones);
}

#[test]
fn one_to_none_shift() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        output: vec![],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            Vec::new(),
            Vec::new(),
        ).expect("rule structure should be valid")),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("a")];

    assert_eq!(Ok(()), apply(&rule, &mut phones, Some(DEFAULT_LINE_APPLICATION_LIMIT)));

    assert_eq!(vec![Phone::Symbol("b")], phones);
}

#[test]
fn remove_word_final_ltr() { 
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        output: vec![],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_any(Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }))]),
            vec![CondPattern::new(CondType::Pattern, PatternList::default(), PatternList::new(vec![Pattern::new_phone(Phone::Bound)]))],
            Vec::new()
        ).expect("rule structure should be valid")),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c"), Phone::Bound, Phone::Symbol("e"), Phone::Bound, Phone::Symbol("f"), Phone::Symbol("g")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, Some(DEFAULT_LINE_APPLICATION_LIMIT)));

    assert_eq!(vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Bound, Phone::Symbol("f")], phones);
}

#[test]
fn remove_word_final_rtl() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Rtl, kind: ShiftType::Move },
        output: vec![],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_any(Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }))]),
            vec![CondPattern::new(CondType::Pattern, PatternList::default(), PatternList::new(vec![Pattern::new_phone(Phone::Bound)]))],
            Vec::new()
        ).expect("rule structure should be valid")),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c"), Phone::Bound, Phone::Symbol("e"), Phone::Bound, Phone::Symbol("f"), Phone::Symbol("g")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, Some(DEFAULT_LINE_APPLICATION_LIMIT)));
    
    assert_eq!(vec![Phone::Bound], phones);
}

#[test]
fn selection_to_selection() {
    let input = PatternList::new(vec![
        Pattern::new_selection(
            vec![
                vec![Pattern::new_phone(Phone::Symbol("a"))],
                vec![Pattern::new_phone(Phone::Symbol("b"))],
                vec![Pattern::new_phone(Phone::Symbol("c"))],
            ],
            Some(ScopeId::IOUnlabeled {
                id_num: 0,
                label_type: LabelType::Scope(ScopeType::Selection),
                parent: None
            }),
        )
    ]);

    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        output: vec![
            Pattern::new_selection(
                vec![
                    vec![Pattern::new_phone(Phone::Symbol("d"))],
                    vec![Pattern::new_phone(Phone::Symbol("e"))],
                    vec![Pattern::new_phone(Phone::Symbol("f"))],
                ],
                Some(ScopeId::IOUnlabeled {
                    id_num: 0,
                    label_type: LabelType::Scope(ScopeType::Selection),
                    parent: None
                }),
            )
        ],
        pattern: RefCell::new(RulePattern::new(input, Vec::new(), Vec::new()).expect("rule structure should be valid")),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c"), Phone::Symbol("d")];

    assert_eq!(Ok(()), apply(&rule, &mut phones, Some(DEFAULT_LINE_APPLICATION_LIMIT)));
    
    assert_eq!(vec![Phone::Symbol("d"), Phone::Symbol("e"), Phone::Symbol("f"), Phone::Symbol("d")], phones);
}

#[test]
fn option_phone_to_option_phone() { 
    let input = PatternList::new(vec![
            Pattern::new_optional(
                vec![Pattern::new_phone(Phone::Symbol("a"))],
                Some(ScopeId::IOUnlabeled {
                    id_num: 0,
                    label_type: LabelType::Scope(ScopeType::Optional),
                    parent: None
                })
            ),
            Pattern::new_phone(Phone::Symbol("b")),
        ]);

    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        output: vec![
            Pattern::new_optional(
                vec![Pattern::new_phone(Phone::Symbol("c"))],
                Some(ScopeId::IOUnlabeled {
                    id_num: 0,
                    label_type: LabelType::Scope(ScopeType::Optional),
                    parent: None
                }),
            ),
            Pattern::new_phone(Phone::Symbol("d")),
        ],
        pattern: RefCell::new(RulePattern::new(
            input,
            Vec::new(),
            Vec::new(),
        ).expect("rule structure should be valid")),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("e"), Phone::Symbol("b"), Phone::Symbol("e")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, Some(DEFAULT_LINE_APPLICATION_LIMIT)));
  
    assert_eq!(vec![Phone::Symbol("c"), Phone::Symbol("d"), Phone::Symbol("e"), Phone::Symbol("d"), Phone::Symbol("e")], phones);
}

#[test]
fn phone_to_phone_word_final_ltr() { 
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        output: vec![Pattern::new_phone(Phone::Symbol("b"))],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            vec![CondPattern::new(CondType::Pattern, PatternList::default(), PatternList::new(vec![Pattern::new_phone(Phone::Bound)]))],
            Vec::new(),
        ).expect("rule structure should be valid")),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("c"), Phone::Symbol("a")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, Some(DEFAULT_LINE_APPLICATION_LIMIT)));
    
    assert_eq!(vec![Phone::Symbol("a"), Phone::Symbol("c"), Phone::Symbol("b")], phones);
}

#[test]
fn phone_to_phone_word_final_rtl() { 
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Rtl, kind: ShiftType::Move },
        output: vec![Pattern::new_phone(Phone::Symbol("b"))],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            vec![CondPattern::new(CondType::Pattern, PatternList::default(), PatternList::new(vec![Pattern::new_phone(Phone::Bound)]))],
            Vec::new(),
        ).expect("rule structure should be valid")),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("c"), Phone::Symbol("a")];

    assert_eq!(Ok(()), apply(&rule, &mut phones, Some(DEFAULT_LINE_APPLICATION_LIMIT)));
    
    assert_eq!(vec![Phone::Symbol("a"), Phone::Symbol("c"), Phone::Symbol("b")], phones);
}

#[test]
fn quadruple_agreement() {
    let input = PatternList::new(vec![
        Pattern::new_selection(
            vec![
                vec![Pattern::new_phone(Phone::Symbol("a"))],
                vec![Pattern::new_phone(Phone::Symbol("b"))],
                vec![Pattern::new_phone(Phone::Symbol("c"))],
            ],
            Some(ScopeId::Name("label")),
    )]);

    let conds = vec![CondPattern::new(CondType::Pattern, PatternList::default(), PatternList::new(vec![
        Pattern::new_selection(
            vec![
                vec![Pattern::new_phone(Phone::Symbol("g"))],
                vec![Pattern::new_phone(Phone::Symbol("h"))],
                vec![Pattern::new_phone(Phone::Symbol("i"))],
            ],
            Some(ScopeId::Name("label")),
        )
    ]))];

    let anti_conds = vec![CondPattern::new(CondType::Pattern, PatternList::new(vec![
        Pattern::new_selection(
            vec![
                vec![Pattern::new_phone(Phone::Symbol("j"))],
                vec![Pattern::new_phone(Phone::Symbol("k"))],
                vec![Pattern::new_phone(Phone::Symbol("l"))],
            ],
            Some(ScopeId::Name("label")),
        )
    ]), PatternList::default() )];

    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        output: vec![
            Pattern::new_selection(
                vec![
                    vec![Pattern::new_phone(Phone::Symbol("d"))],
                    vec![Pattern::new_phone(Phone::Symbol("e"))],
                    vec![Pattern::new_phone(Phone::Symbol("f"))],
                ],
                Some(ScopeId::Name("label")),
            )
        ],
        pattern: RefCell::new(RulePattern::new(input, conds, anti_conds).expect("rule structure should be valid")),
    };

    let mut phones = vec![
        Phone::Symbol("a"),
        Phone::Symbol("g"),
        Phone::Symbol("a"),
        Phone::Symbol("h"),
        Phone::Symbol("b"),
        Phone::Symbol("h"),
        Phone::Symbol("j"),
        Phone::Symbol("c"),
        Phone::Symbol("i"),
        Phone::Symbol("l"),
        Phone::Symbol("c"),
        Phone::Symbol("i"),
    ];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, Some(DEFAULT_LINE_APPLICATION_LIMIT)));
    
    assert_eq!(vec![
        Phone::Symbol("d"),
        Phone::Symbol("g"),
        Phone::Symbol("a"),
        Phone::Symbol("h"),
        Phone::Symbol("e"),
        Phone::Symbol("h"),
        Phone::Symbol("j"),
        Phone::Symbol("f"),
        Phone::Symbol("i"),
        Phone::Symbol("l"),
        Phone::Symbol("c"),
        Phone::Symbol("i")
    ], phones);
}

#[test]
fn count_limit() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        output: vec![Pattern::new_phone(Phone::Symbol("b"))],
        pattern: RefCell::new(RulePattern::new(
            PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
            Vec::new(),
        Vec::new()
        ).expect("rule structure should be valid")),
    };

    assert!(apply(&rule, &mut vec![Phone::Symbol("a")], Some(LineApplicationLimit::Attempts(1))).is_ok());
    assert!(apply(&rule, &mut vec![Phone::Symbol("a"), Phone::Symbol("b")], Some(LineApplicationLimit::Attempts(1))).is_err());
}