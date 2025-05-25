use crate::{rules::{conditions::{Cond, CondType}, tokens::{LabelType, ScopeId}}, runtime::DEFAULT_LINE_APPLICATION_LIMIT, tokens::{ScopeType, Shift}};
use super::*;

#[test]
fn apply_empty_rule_to_no_phones() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        input: Vec::new(),
        output: Vec::new(),
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };
    
    assert_eq!(Ok(()), apply(&rule, &mut Vec::new(), DEFAULT_LINE_APPLICATION_LIMIT));
}

#[test]
fn apply_empty_rule_to_one_phone() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        input: Vec::new(),
        output: Vec::new(),
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };
    
    assert_eq!(Err(ApplicationError::MatchError(MatchError::EmptyInput)), apply(&rule, &mut vec![Phone::Symbol("a")], DEFAULT_LINE_APPLICATION_LIMIT));
}

#[test]
fn one_to_one_shift() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: vec![RuleToken::Phone(Phone::Symbol("b"))],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("c"), Phone::Symbol("a")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, DEFAULT_LINE_APPLICATION_LIMIT));

    assert_eq!(vec![Phone::Symbol("b"), Phone::Symbol("c"), Phone::Symbol("b")], phones);
}

#[test]
fn one_to_two_shift() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: vec![
            RuleToken::Phone(Phone::Symbol("b")),
            RuleToken::Phone(Phone::Symbol("c")),
        ],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("d"), Phone::Symbol("a")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, DEFAULT_LINE_APPLICATION_LIMIT));

    assert_eq!(vec![Phone::Symbol("b"), Phone::Symbol("c"), Phone::Symbol("d"), Phone::Symbol("b"), Phone::Symbol("c")], phones);
}

#[test]
fn two_to_one_shift() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        input: vec![
            RuleToken::Phone(Phone::Symbol("a")),
            RuleToken::Phone(Phone::Symbol("b")),
        ],
        output: vec![RuleToken::Phone(Phone::Symbol("c"))],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("d"), Phone::Symbol("a"), Phone::Symbol("b")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, DEFAULT_LINE_APPLICATION_LIMIT));

    assert_eq!(vec![Phone::Symbol("c"), Phone::Symbol("d"), Phone::Symbol("c")], phones);
}

#[test]
fn one_to_none_shift() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: vec![],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("a")];

    assert_eq!(Ok(()), apply(&rule, &mut phones, DEFAULT_LINE_APPLICATION_LIMIT));

    assert_eq!(vec![Phone::Symbol("b")], phones);
}

#[test]
fn remove_word_final_ltr() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        input: vec![RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) }],
        output: vec![],
        conds: vec![Cond::new(CondType::Pattern, Vec::new(), vec![RuleToken::Phone(Phone::Bound)])],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c"), Phone::Bound, Phone::Symbol("e"), Phone::Bound, Phone::Symbol("f"), Phone::Symbol("g")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, DEFAULT_LINE_APPLICATION_LIMIT));

    assert_eq!(vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Bound, Phone::Symbol("f")], phones);
}

#[test]
fn remove_word_final_rtl() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Rtl, kind: ShiftType::Move },
        input: vec![RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) }],
        output: vec![],
        conds: vec![Cond::new(CondType::Pattern, Vec::new(), vec![RuleToken::Phone(Phone::Bound)])],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c"), Phone::Bound, Phone::Symbol("e"), Phone::Bound, Phone::Symbol("f"), Phone::Symbol("g")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, DEFAULT_LINE_APPLICATION_LIMIT));
    
    assert_eq!(vec![Phone::Bound], phones);
}

#[test]
fn selection_to_selection() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        input: vec![
            RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled {
                id_num: 0,
                label_type: LabelType::Scope(ScopeType::Selection),
                parent: None
            }), options: vec![
                vec![RuleToken::Phone(Phone::Symbol("a"))],
                vec![RuleToken::Phone(Phone::Symbol("b"))],
                vec![RuleToken::Phone(Phone::Symbol("c"))],
            ] }
        ],
        output: vec![
            RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled {
                id_num: 0,
                label_type: LabelType::Scope(ScopeType::Selection),
                parent: None
            }), options: vec![
                vec![RuleToken::Phone(Phone::Symbol("d"))],
                vec![RuleToken::Phone(Phone::Symbol("e"))],
                vec![RuleToken::Phone(Phone::Symbol("f"))],
            ] }
        ],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c"), Phone::Symbol("d")];

    assert_eq!(Ok(()), apply(&rule, &mut phones, DEFAULT_LINE_APPLICATION_LIMIT));
    
    assert_eq!(vec![Phone::Symbol("d"), Phone::Symbol("e"), Phone::Symbol("f"), Phone::Symbol("d")], phones);
}

#[test]
fn option_to_phone() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        input: vec![
            RuleToken::OptionalScope { id: Some(ScopeId::IOUnlabeled {
                id_num: 0,
                label_type: LabelType::Scope(ScopeType::Optional),
                parent: None
            }), content: vec![RuleToken::Phone(Phone::Symbol("a"))] },
        ],
        output: vec![
            RuleToken::Phone(Phone::Symbol("b")),
        ],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")];
    
    assert_eq!(Err(ApplicationError::MatchError(MatchError::EmptyInput)), apply(&rule, &mut phones, DEFAULT_LINE_APPLICATION_LIMIT));
}

#[test]
fn option_phone_to_option_phone() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        input: vec![
            RuleToken::OptionalScope { id: Some(ScopeId::IOUnlabeled {
                id_num: 0,
                label_type: LabelType::Scope(ScopeType::Optional),
                parent: None
            }), content: vec![RuleToken::Phone(Phone::Symbol("a"))] },
            RuleToken::Phone(Phone::Symbol("b")),
        ],
        output: vec![
            RuleToken::OptionalScope { id: Some(ScopeId::IOUnlabeled {
                id_num: 0,
                label_type: LabelType::Scope(ScopeType::Optional),
                parent: None
            }), content: vec![RuleToken::Phone(Phone::Symbol("c"))] },
            RuleToken::Phone(Phone::Symbol("d")),
        ],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("e"), Phone::Symbol("b"), Phone::Symbol("e")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, DEFAULT_LINE_APPLICATION_LIMIT,  ));
  
    assert_eq!(vec![Phone::Symbol("c"), Phone::Symbol("d"), Phone::Symbol("e"), Phone::Symbol("d"), Phone::Symbol("e")], phones);
}

#[test]
fn phone_to_phone_word_final_ltr() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: vec![RuleToken::Phone(Phone::Symbol("b"))],
        conds: vec![Cond::new(CondType::Pattern, Vec::new(), vec![RuleToken::Phone(Phone::Bound)])],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("c"), Phone::Symbol("a")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, DEFAULT_LINE_APPLICATION_LIMIT));
    
    assert_eq!(vec![Phone::Symbol("a"), Phone::Symbol("c"), Phone::Symbol("b")], phones);
}

#[test]
fn phone_to_phone_word_final_rtl() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Rtl, kind: ShiftType::Move },
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: vec![RuleToken::Phone(Phone::Symbol("b"))],
        conds: vec![Cond::new(CondType::Pattern, Vec::new(), vec![RuleToken::Phone(Phone::Bound)])],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::Symbol("a"), Phone::Symbol("c"), Phone::Symbol("a")];

    assert_eq!(Ok(()), apply(&rule, &mut phones, DEFAULT_LINE_APPLICATION_LIMIT));
    
    assert_eq!(vec![Phone::Symbol("a"), Phone::Symbol("c"), Phone::Symbol("b")], phones);
}

#[test]
fn quadruple_agreement() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::Ltr, kind: ShiftType::Move },
        input: vec![
            RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
                vec![RuleToken::Phone(Phone::Symbol("a"))],
                vec![RuleToken::Phone(Phone::Symbol("b"))],
                vec![RuleToken::Phone(Phone::Symbol("c"))],
            ] }
        ],
        output: vec![
            RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
                vec![RuleToken::Phone(Phone::Symbol("d"))],
                vec![RuleToken::Phone(Phone::Symbol("e"))],
                vec![RuleToken::Phone(Phone::Symbol("f"))],
            ] }
        ],
        conds: vec![Cond::new(CondType::Pattern, Vec::new(), vec![
            RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
                vec![RuleToken::Phone(Phone::Symbol("g"))],
                vec![RuleToken::Phone(Phone::Symbol("h"))],
                vec![RuleToken::Phone(Phone::Symbol("i"))],
            ] }
        ] )],
        anti_conds: vec![Cond::new(CondType::Pattern, vec![
            RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
                vec![RuleToken::Phone(Phone::Symbol("j"))],
                vec![RuleToken::Phone(Phone::Symbol("k"))],
                vec![RuleToken::Phone(Phone::Symbol("l"))],
            ] }
        ], Vec::new() )],
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
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, DEFAULT_LINE_APPLICATION_LIMIT));
    
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
        input: vec![RuleToken::Phone(Phone::Symbol("a"))],
        output: vec![RuleToken::Phone(Phone::Symbol("b"))],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    assert!(apply(&rule, &mut vec![Phone::Symbol("a")], LineApplicationLimit::Attempts(1)).is_ok());
    assert!(apply(&rule, &mut vec![Phone::Symbol("a"), Phone::Symbol("b")], LineApplicationLimit::Attempts(1)).is_err());
}