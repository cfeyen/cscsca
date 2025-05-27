use super::*;

#[test]
fn match_input() {
    let cond = Cond::new(
        CondType::Pattern,
        vec!(RuleToken::Phone(Phone::Symbol("a"))),
        vec!(RuleToken::Phone(Phone::Symbol("c"))),
    );

    assert!(cond.eval(
        &[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")],
        1,
        1, 
        &mut Choices::default(),
        Direction::Ltr,
    ).unwrap());

    assert!(cond.eval(
        &[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")],
        1,
        1, 
        &mut Choices::default(),
        Direction::Rtl,
    ).unwrap());

    assert!(!cond.eval(
        &[Phone::Symbol("a"), Phone::Symbol("b")],
        1,
        1, 
        &mut Choices::default(),
        Direction::Ltr,
    ).unwrap());
}

#[test]
fn equality() {
    assert!(Cond::new(
        CondType::Match,
        vec![RuleToken::Phone(Phone::Symbol("a"))],
        vec![RuleToken::Phone(Phone::Symbol("a"))]
    ).eval(&[], 0, 0, &mut Choices::default(), Direction::Ltr).unwrap());

    assert!(!Cond::new(
        CondType::Match,
        vec![RuleToken::Phone(Phone::Symbol("a"))],
        vec![RuleToken::Phone(Phone::Symbol("b"))]
    ).eval(&[], 0, 0, &mut Choices::default(), Direction::Ltr).unwrap());

    assert!(!Cond::new(
        CondType::Match,
        vec![RuleToken::Phone(Phone::Symbol("a a"))],
        vec![RuleToken::Phone(Phone::Symbol("a")), RuleToken::Phone(Phone::Symbol("a"))]
    ).eval(&[], 0, 0, &mut Choices::default(), Direction::Ltr).unwrap());
}

#[test]
fn and() {
    let mut cond = Cond::new(
        CondType::Pattern,
        vec![RuleToken::Phone(Phone::Symbol("b"))],
        vec![RuleToken::Phone(Phone::Symbol("d"))],
    );

    let phones_1 = &[
        Phone::Symbol("a"),
        Phone::Symbol("b"),
        Phone::Symbol("c"),
        Phone::Symbol("d"),
        Phone::Symbol("e"),
    ];

    let phones_2 = &[
        Phone::Symbol("a"),
        Phone::Symbol("b"),
        Phone::Symbol("c"),
        Phone::Symbol("d"),
        Phone::Symbol("f"),
    ];

    assert!(cond.eval(phones_1, 2, 1, &mut Choices::default(), Direction::Ltr).unwrap());

    assert!(cond.eval(phones_2, 2, 1, &mut Choices::default(), Direction::Ltr).unwrap());

    cond.set_and(AndType::And, Cond::new(
        CondType::Pattern,
        vec![RuleToken::Phone(Phone::Symbol("a")), RuleToken::Any { id: None }],
        vec![RuleToken::Any { id: None }, RuleToken::Phone(Phone::Symbol("e"))],
    ));

    assert!(cond.eval(phones_1, 2, 1, &mut Choices::default(), Direction::Ltr).unwrap());

    assert!(!cond.eval(phones_2, 2, 1, &mut Choices::default(), Direction::Ltr).unwrap());
}

#[test]
fn and_not() {
    let mut cond = Cond::new(
        CondType::Pattern,
        vec![RuleToken::Phone(Phone::Symbol("b"))],
        vec![RuleToken::Phone(Phone::Symbol("d"))],
    );

    let phones_1 = &[
        Phone::Symbol("a"),
        Phone::Symbol("b"),
        Phone::Symbol("c"),
        Phone::Symbol("d"),
        Phone::Symbol("e"),
    ];

    let phones_2 = &[
        Phone::Symbol("a"),
        Phone::Symbol("b"),
        Phone::Symbol("c"),
        Phone::Symbol("d"),
        Phone::Symbol("f"),
    ];

    assert!(cond.eval(phones_1, 2, 1, &mut Choices::default(), Direction::Ltr).unwrap());

    assert!(cond.eval(phones_2, 2, 1, &mut Choices::default(), Direction::Ltr).unwrap());

    cond.set_and(AndType::AndNot, Cond::new(
        CondType::Pattern,
        vec![RuleToken::Phone(Phone::Symbol("a")), RuleToken::Any { id: None }],
        vec![RuleToken::Any { id: None }, RuleToken::Phone(Phone::Symbol("e"))],
    ));

    assert!(!cond.eval(phones_1, 2, 1, &mut Choices::default(), Direction::Ltr).unwrap());

    assert!(cond.eval(phones_2, 2, 1, &mut Choices::default(), Direction::Ltr).unwrap());
}