use crate::{matcher::{patterns::{cond::CondPattern, list::PatternList, rule::RulePattern, Pattern}, phones::Phones}, phones::Phone, tokens::{AndType, CondType, Direction, ScopeId}};

#[test]
fn matches_phones() {
    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        Vec::new(),
        Vec::new()
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![
            Pattern::new_phone(Phone::Symbol("a")),
            Pattern::new_phone(Phone::Symbol("b")),
            Pattern::new_phone(Phone::Symbol("c")),
        ]),
        Vec::new(),
        Vec::new()
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")], 2, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_some());
}

#[test]
fn match_phone_with_cond() {
    let before_b = vec![CondPattern::new(CondType::Pattern, PatternList::default(), PatternList::new(vec![Pattern::new_phone(Phone::Symbol("b"))]))];

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        before_b.clone(),
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        before_b.clone(),
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        before_b,
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());

    let after_b = vec![CondPattern::new(CondType::Pattern, PatternList::new(vec![Pattern::new_phone(Phone::Symbol("b"))]), PatternList::default())];

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        after_b.clone(),
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a")], 1, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        after_b.clone(),
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        after_b,
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b")], 1, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());
}

#[test]
fn match_phone_with_anti_cond() {
    let before_b = vec![CondPattern::new(CondType::Pattern, PatternList::default(), PatternList::new(vec![Pattern::new_phone(Phone::Symbol("b"))]))];

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        Vec::new(),
        before_b.clone(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        Vec::new(),
        before_b.clone(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        Vec::new(),
        before_b,
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());

    let after_b = vec![CondPattern::new(CondType::Pattern, PatternList::new(vec![Pattern::new_phone(Phone::Symbol("b"))]), PatternList::default())];

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        Vec::new(),
        after_b.clone(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a")], 1, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        Vec::new(),
        after_b.clone(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        Vec::new(),
        after_b,
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());
}

#[test]
fn and_cond() {
    let mut between_b_and_c = CondPattern::new(CondType::Pattern, PatternList::new(vec![Pattern::new_phone(Phone::Symbol("b"))]), PatternList::default());
    let before_c = CondPattern::new(CondType::Pattern, PatternList::default(), PatternList::new(vec![Pattern::new_phone(Phone::Symbol("c"))]));

    between_b_and_c.add_and(AndType::And, before_c);

    let cond = vec![between_b_and_c];

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        cond.clone(),
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a"), Phone::Symbol("c")], 1, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        cond.clone(),
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("c")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        cond.clone(),
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a")], 1, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        cond,
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());
}

#[test]
fn and_not_cond() {
    let mut between_b_and_c = CondPattern::new(CondType::Pattern, PatternList::new(vec![Pattern::new_phone(Phone::Symbol("b"))]), PatternList::default());
    let before_c = CondPattern::new(CondType::Pattern, PatternList::default(), PatternList::new(vec![Pattern::new_phone(Phone::Symbol("c"))]));

    between_b_and_c.add_and(AndType::AndNot, before_c);

    let cond = vec![between_b_and_c];

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        cond.clone(),
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a"), Phone::Symbol("c")], 1, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        cond.clone(),
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("c")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        cond.clone(),
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a")], 1, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        cond,
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());
}

#[test]
fn agreement_between_pattern_halves() {
    let cond = vec![CondPattern::new(
        CondType::Pattern,
        PatternList::new(vec![
            Pattern::new_selection(
                vec![
                    vec![Pattern::new_phone(Phone::Symbol("c"))],
                    vec![Pattern::new_phone(Phone::Symbol("d"))],
                ],
                Some(ScopeId::Name("label"))
            )
        ]),
        PatternList::new(vec![
            Pattern::new_selection(
                vec![
                    vec![Pattern::new_phone(Phone::Symbol("c"))],
                    vec![Pattern::new_phone(Phone::Symbol("d"))],
                ],
                Some(ScopeId::Name("label"))
            )
        ])
    )];

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        cond.clone(),
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("c"), Phone::Symbol("a"), Phone::Symbol("c")], 1, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        cond,
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("c"), Phone::Symbol("a"), Phone::Symbol("d")], 1, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());
}

#[test]
fn agreement_between_and_conds() {
    let mut cond = CondPattern::new(
        CondType::Pattern,
        PatternList::new(vec![
            Pattern::new_selection(
                vec![
                    vec![Pattern::new_phone(Phone::Symbol("c"))],
                    vec![Pattern::new_phone(Phone::Symbol("d"))],
                ],
                Some(ScopeId::Name("label")),
            )
        ]),
        PatternList::default()
    );

    cond.add_and(AndType::And, CondPattern::new(
        CondType::Pattern,
        PatternList::default(), 
        PatternList::new(vec![
            Pattern::new_selection(
                vec![
                    vec![Pattern::new_phone(Phone::Symbol("c"))],
                    vec![Pattern::new_phone(Phone::Symbol("d"))],
                ],
                Some(ScopeId::Name("label")),
            )
        ])
    ));

    let cond = vec![cond];

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        cond.clone(),
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("c"), Phone::Symbol("a"), Phone::Symbol("c")], 1, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        cond,
        Vec::new(),
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("c"), Phone::Symbol("a"), Phone::Symbol("d")], 1, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_none());
}

#[test]
fn complex_argeement() {
    let scope_id = ScopeId::Name("c");

    let cond = vec![CondPattern::new(CondType::Pattern, PatternList::default(), PatternList::new(vec![
        Pattern::new_selection(
            vec![
                vec![Pattern::new_phone(Phone::Symbol("b"))],
                vec![Pattern::new_any(None), Pattern::new_phone(Phone::Symbol("c"))],
            ],
            Some(scope_id.clone())
        )
    ]))];

    let anti_cond = vec![CondPattern::new(CondType::Pattern, PatternList::default(), PatternList::new(vec![
        Pattern::new_selection(
            vec![
                vec![Pattern::new_phone(Phone::Symbol("b"))],
                vec![Pattern::new_phone(Phone::Symbol("d"))],
            ],
            Some(scope_id)
        )
    ]))];

    let mut rule_pattern = RulePattern::new(
        PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]),
        cond,
        anti_cond,
    ).expect("pattern construction should be valid");

    let match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).expect("next match should not error").is_some());
}

#[test]
fn phone_match_phone_cond() {
    let conds = vec![CondPattern::new(CondType::Match, PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]))];

    let mut rule_pattern = RulePattern::new(PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), conds, Vec::new()).expect("pattern construction should be valid");

    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).is_ok_and(|res| res.is_some()));


    let bad_conds = vec![CondPattern::new(CondType::Match, PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), PatternList::new(vec![Pattern::new_phone(Phone::Symbol("b"))]))];

    let mut rule_pattern = RulePattern::new(PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), bad_conds, Vec::new()).expect("pattern construction should be valid");

    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).is_ok_and(|res| res.is_none()));
}

#[test]
fn optional_match_conds() {
    let label = ScopeId::Name("label");

    let conds = vec![CondPattern::new(CondType::Match, PatternList::new(vec![Pattern::new_optional(vec![Pattern::new_phone(Phone::Symbol("a"))], Some(label.clone()))]), PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]))];

    let mut rule_pattern = RulePattern::new(PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), conds.clone(), Vec::new()).expect("pattern construction should be valid");

    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).is_err());

    let input = PatternList::new(vec![Pattern::new_optional(vec![Pattern::new_phone(Phone::Symbol("a"))], Some(label.clone()))]);

    let mut rule_pattern = RulePattern::new(input, conds, Vec::new()).expect("pattern construction should be valid");

    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).is_ok_and(|res| res.is_some()));

    let conds = vec![CondPattern::new(CondType::Match, PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), PatternList::new(vec![Pattern::new_optional(vec![Pattern::new_phone(Phone::Symbol("a"))], None)]))];

    let mut rule_pattern = RulePattern::new(PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), conds, Vec::new()).expect("pattern construction should be valid");

    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).is_ok_and(|res| res.is_some()));
}

#[test]
fn selection_match_conds() {
    let label = ScopeId::Name("label");

    let conds = vec![CondPattern::new(CondType::Match, PatternList::new(vec![Pattern::new_selection(vec![vec![Pattern::new_phone(Phone::Symbol("a"))], vec![Pattern::new_phone(Phone::Symbol("b"))]], Some(label.clone()))]), PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]))];

    let mut rule_pattern = RulePattern::new(PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), conds.clone(), Vec::new()).expect("pattern construction should be valid");

    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).is_err());

    let input = PatternList::new(vec![Pattern::new_selection(vec![vec![Pattern::new_phone(Phone::Symbol("a"))], vec![Pattern::new_phone(Phone::Symbol("b"))]], Some(label.clone()))]);

    let mut rule_pattern = RulePattern::new(input, conds, Vec::new()).expect("pattern construction should be valid");

    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).is_ok_and(|res| res.is_some()));

    let conds = vec![CondPattern::new(CondType::Match, PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), PatternList::new(vec![Pattern::new_selection(vec![vec![Pattern::new_phone(Phone::Symbol("a"))], vec![Pattern::new_phone(Phone::Symbol("b"))]], None)]))];

    let mut rule_pattern = RulePattern::new(PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), conds, Vec::new()).expect("pattern construction should be valid");

    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).is_ok_and(|res| res.is_some()));
}

#[test]
fn inequal_length_match_conds() {
    let conds = vec![CondPattern::new(CondType::Match, PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a")), Pattern::new_phone(Phone::Symbol("b"))]), PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]))];
    
    let mut rule_pattern = RulePattern::new(PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), conds, Vec::new()).expect("pattern construction should be valid");

    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).is_ok_and(|res| res.is_none()));

    let conds = vec![CondPattern::new(CondType::Match, PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a")), Pattern::new_phone(Phone::Symbol("b"))]))];
    
    let mut rule_pattern = RulePattern::new(PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), conds, Vec::new()).expect("pattern construction should be valid");

    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(rule_pattern.next_match(&match_phones).is_ok_and(|res| res.is_none()));
}

#[test]
fn zero_input() {
    let mut rule_pattern = RulePattern::new(PatternList::default(), Vec::new(), Vec::new()).expect("pattern construction should be valid");

    assert!(rule_pattern.next_match(&Phones::new(&[], 0, Direction::Rtl)).expect("next match should not error").is_some());
    assert!(rule_pattern.next_match(&Phones::new(&[], 0, Direction::Rtl)).expect("next match should not error").is_none());


    let cond = vec![CondPattern::new(CondType::Pattern, PatternList::new(vec![Pattern::new_phone(Phone::Bound)]), PatternList::new(vec![Pattern::new_phone(Phone::Bound)]))];
    let mut rule_pattern = RulePattern::new(PatternList::default(), cond, Vec::new()).expect("pattern construction should be valid");

    assert!(rule_pattern.next_match(&Phones::new(&[], 0, Direction::Rtl)).expect("next match should not error").is_some());
    assert!(rule_pattern.next_match(&Phones::new(&[], 0, Direction::Rtl)).expect("next match should not error").is_none());
    
    let cond = vec![CondPattern::new(CondType::Pattern, PatternList::new(vec![Pattern::new_phone(Phone::Symbol("a"))]), PatternList::default())];
    let mut rule_pattern = RulePattern::new(PatternList::default(), cond, Vec::new()).expect("pattern construction should be valid");

    assert!(rule_pattern.next_match(&Phones::new(&[], 0, Direction::Rtl)).expect("next match should not error").is_none());
}

// todo: conds, anti-conds, &, &!, with gaps, non phone conds