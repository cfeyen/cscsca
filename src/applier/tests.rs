use crate::{meta_tokens::ScopeType, rules::{conditions::{Cond, CondType}, sound_change_rule::{LabelType, ScopeId}}};
use super::*;

use crate::runtime::DEFAULT_MAX_APPLICATION_TIME;

#[test]
fn apply_empty_rule_to_no_phones() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::LTR, kind: ShiftType::Move },
        input: Vec::new(),
        output: Vec::new(),
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };
    
    assert_eq!(Ok(()), apply(&rule, &mut Vec::new(), &DEFAULT_MAX_APPLICATION_TIME));
}

#[test]
fn apply_empty_rule_to_one_phone() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::LTR, kind: ShiftType::Move },
        input: Vec::new(),
        output: Vec::new(),
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };
    
    assert_eq!(Err(ApplicationError::MatchError(MatchError::EmptyInput)), apply(&rule, &mut vec![Phone::new("a")], &DEFAULT_MAX_APPLICATION_TIME));
}

#[test]
fn one_to_one_shift() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::LTR, kind: ShiftType::Move },
        input: vec![RuleToken::Phone(Phone::new("a"))],
        output: vec![RuleToken::Phone(Phone::new("b"))],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::new("a"), Phone::new("c"), Phone::new("a")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, &DEFAULT_MAX_APPLICATION_TIME));

    assert_eq!(vec![Phone::new("b"), Phone::new("c"), Phone::new("b")], phones);
}

#[test]
fn one_to_two_shift() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::LTR, kind: ShiftType::Move },
        input: vec![RuleToken::Phone(Phone::new("a"))],
        output: vec![
            RuleToken::Phone(Phone::new("b")),
            RuleToken::Phone(Phone::new("c")),
        ],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::new("a"), Phone::new("d"), Phone::new("a")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, &DEFAULT_MAX_APPLICATION_TIME));

    assert_eq!(vec![Phone::new("b"), Phone::new("c"), Phone::new("d"), Phone::new("b"), Phone::new("c")], phones);
}

#[test]
fn two_to_one_shift() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::LTR, kind: ShiftType::Move },
        input: vec![
            RuleToken::Phone(Phone::new("a")),
            RuleToken::Phone(Phone::new("b")),
        ],
        output: vec![RuleToken::Phone(Phone::new("c"))],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::new("a"), Phone::new("b"), Phone::new("d"), Phone::new("a"), Phone::new("b")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, &DEFAULT_MAX_APPLICATION_TIME));

    assert_eq!(vec![Phone::new("c"), Phone::new("d"), Phone::new("c")], phones);
}

#[test]
fn one_to_none_shift() {
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::LTR, kind: ShiftType::Move },
        input: vec![RuleToken::Phone(Phone::new("a"))],
        output: vec![],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::new("a"), Phone::new("b"), Phone::new("a")];

    assert_eq!(Ok(()), apply(&rule, &mut phones, &DEFAULT_MAX_APPLICATION_TIME));

    assert_eq!(vec![Phone::new("b")], phones);
}

#[test]
fn remove_word_final_ltr() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::LTR, kind: ShiftType::Move },
        input: vec![RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) }],
        output: vec![],
        conds: vec![Cond::new(CondType::MatchInput, Vec::new(), vec![RuleToken::Phone(Phone::new_bound())])],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::new("a"), Phone::new("b"), Phone::new("c"), Phone::new_bound(), Phone::new("e"), Phone::new_bound(), Phone::new("f"), Phone::new("g")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, &DEFAULT_MAX_APPLICATION_TIME));

    assert_eq!(vec![Phone::new("a"), Phone::new("b"), Phone::new_bound(), Phone::new("f")], phones);
}

#[test]
fn remove_word_final_rtl() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::RTL, kind: ShiftType::Move },
        input: vec![RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) }],
        output: vec![],
        conds: vec![Cond::new(CondType::MatchInput, Vec::new(), vec![RuleToken::Phone(Phone::new_bound())])],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::new("a"), Phone::new("b"), Phone::new("c"), Phone::new_bound(), Phone::new("e"), Phone::new_bound(), Phone::new("f"), Phone::new("g")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, &DEFAULT_MAX_APPLICATION_TIME));
    
    assert_eq!(vec![Phone::new_bound()], phones);
}

#[test]
fn selection_to_selection() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::LTR, kind: ShiftType::Move },
        input: vec![
            RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled {
                id_num: 0,
                label_type: LabelType::Scope(ScopeType::Selection),
                parent: None
            }), options: vec![
                vec![RuleToken::Phone(Phone::new("a"))],
                vec![RuleToken::Phone(Phone::new("b"))],
                vec![RuleToken::Phone(Phone::new("c"))],
            ] }
        ],
        output: vec![
            RuleToken::SelectionScope { id: Some(ScopeId::IOUnlabeled {
                id_num: 0,
                label_type: LabelType::Scope(ScopeType::Selection),
                parent: None
            }), options: vec![
                vec![RuleToken::Phone(Phone::new("d"))],
                vec![RuleToken::Phone(Phone::new("e"))],
                vec![RuleToken::Phone(Phone::new("f"))],
            ] }
        ],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::new("a"), Phone::new("b"), Phone::new("c"), Phone::new("d")];

    assert_eq!(Ok(()), apply(&rule, &mut phones, &DEFAULT_MAX_APPLICATION_TIME));
    
    assert_eq!(vec![Phone::new("d"), Phone::new("e"), Phone::new("f"), Phone::new("d")], phones);
}

#[test]
fn option_to_phone() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::LTR, kind: ShiftType::Move },
        input: vec![
            RuleToken::OptionalScope { id: Some(ScopeId::IOUnlabeled {
                id_num: 0,
                label_type: LabelType::Scope(ScopeType::Optional),
                parent: None
            }), content: vec![RuleToken::Phone(Phone::new("a"))] },
        ],
        output: vec![
            RuleToken::Phone(Phone::new("b")),
        ],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::new("a"), Phone::new("b"), Phone::new("c")];
    
    assert_eq!(Err(ApplicationError::MatchError(MatchError::EmptyInput)), apply(&rule, &mut phones, &DEFAULT_MAX_APPLICATION_TIME));
}

#[test]
fn option_phone_to_option_phone() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::LTR, kind: ShiftType::Move },
        input: vec![
            RuleToken::OptionalScope { id: Some(ScopeId::IOUnlabeled {
                id_num: 0,
                label_type: LabelType::Scope(ScopeType::Optional),
                parent: None
            }), content: vec![RuleToken::Phone(Phone::new("a"))] },
            RuleToken::Phone(Phone::new("b")),
        ],
        output: vec![
            RuleToken::OptionalScope { id: Some(ScopeId::IOUnlabeled {
                id_num: 0,
                label_type: LabelType::Scope(ScopeType::Optional),
                parent: None
            }), content: vec![RuleToken::Phone(Phone::new("c"))] },
            RuleToken::Phone(Phone::new("d")),
        ],
        conds: vec![Cond::default()],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::new("a"), Phone::new("b"), Phone::new("e"), Phone::new("b"), Phone::new("e")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, &DEFAULT_MAX_APPLICATION_TIME  ));
  
    assert_eq!(vec![Phone::new("c"), Phone::new("d"), Phone::new("e"), Phone::new("d"), Phone::new("e")], phones);
}

#[test]
fn phone_to_phone_word_final_ltr() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::LTR, kind: ShiftType::Move },
        input: vec![RuleToken::Phone(Phone::new("a"))],
        output: vec![RuleToken::Phone(Phone::new("b"))],
        conds: vec![Cond::new(CondType::MatchInput, Vec::new(), vec![RuleToken::Phone(Phone::new_bound())])],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::new("a"), Phone::new("c"), Phone::new("a")];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, &DEFAULT_MAX_APPLICATION_TIME));
    
    assert_eq!(vec![Phone::new("a"), Phone::new("c"), Phone::new("b")], phones);
}

#[test]
fn phone_to_phone_word_final_rtl() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::RTL, kind: ShiftType::Move },
        input: vec![RuleToken::Phone(Phone::new("a"))],
        output: vec![RuleToken::Phone(Phone::new("b"))],
        conds: vec![Cond::new(CondType::MatchInput, Vec::new(), vec![RuleToken::Phone(Phone::new_bound())])],
        anti_conds: Vec::new(),
    };

    let mut phones = vec![Phone::new("a"), Phone::new("c"), Phone::new("a")];

    assert_eq!(Ok(()), apply(&rule, &mut phones, &DEFAULT_MAX_APPLICATION_TIME));
    
    assert_eq!(vec![Phone::new("a"), Phone::new("c"), Phone::new("b")], phones);
}

#[test]
fn quadruple_agreement() { // also tests bound deduplication
    let rule = SoundChangeRule {
        kind: Shift { dir: Direction::LTR, kind: ShiftType::Move },
        input: vec![
            RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
                vec![RuleToken::Phone(Phone::new("a"))],
                vec![RuleToken::Phone(Phone::new("b"))],
                vec![RuleToken::Phone(Phone::new("c"))],
            ] }
        ],
        output: vec![
            RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
                vec![RuleToken::Phone(Phone::new("d"))],
                vec![RuleToken::Phone(Phone::new("e"))],
                vec![RuleToken::Phone(Phone::new("f"))],
            ] }
        ],
        conds: vec![Cond::new(CondType::MatchInput, Vec::new(), vec![
            RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
                vec![RuleToken::Phone(Phone::new("g"))],
                vec![RuleToken::Phone(Phone::new("h"))],
                vec![RuleToken::Phone(Phone::new("i"))],
            ] }
        ] )],
        anti_conds: vec![Cond::new(CondType::MatchInput, vec![
            RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
                vec![RuleToken::Phone(Phone::new("j"))],
                vec![RuleToken::Phone(Phone::new("k"))],
                vec![RuleToken::Phone(Phone::new("l"))],
            ] }
        ], Vec::new() )],
    };

    let mut phones = vec![
        Phone::new("a"),
        Phone::new("g"),
        Phone::new("a"),
        Phone::new("h"),
        Phone::new("b"),
        Phone::new("h"),
        Phone::new("j"),
        Phone::new("c"),
        Phone::new("i"),
        Phone::new("l"),
        Phone::new("c"),
        Phone::new("i"),
    ];
    
    assert_eq!(Ok(()), apply(&rule, &mut phones, &DEFAULT_MAX_APPLICATION_TIME));
    
    assert_eq!(vec![
        Phone::new("d"),
        Phone::new("g"),
        Phone::new("a"),
        Phone::new("h"),
        Phone::new("e"),
        Phone::new("h"),
        Phone::new("j"),
        Phone::new("f"),
        Phone::new("i"),
        Phone::new("l"),
        Phone::new("c"),
        Phone::new("i")
    ], phones);
}