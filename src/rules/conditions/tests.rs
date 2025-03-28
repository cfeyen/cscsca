use super::*;

#[test]
fn match_input() {
    let cond = Cond::new(
        CondType::MatchInput,
        vec!(RuleToken::Phone(Phone::new("a"))),
        vec!(RuleToken::Phone(Phone::new("c"))),
    );

    assert!(cond.eval(
        &[Phone::new("a"), Phone::new("b"), Phone::new("c")],
        1,
        1, 
        &mut Choices::default(),
        Direction::LTR,
    ).unwrap());

    assert!(cond.eval(
        &[Phone::new("a"), Phone::new("b"), Phone::new("c")],
        1,
        1, 
        &mut Choices::default(),
        Direction::RTL,
    ).unwrap());

    assert!(!cond.eval(
        &[Phone::new("a"), Phone::new("b")],
        1,
        1, 
        &mut Choices::default(),
        Direction::LTR,
    ).unwrap());
}

#[test]
fn equality() {
    assert!(Cond::new(
        CondType::Equality,
        vec![RuleToken::Phone(Phone::new("a"))],
        vec![RuleToken::Phone(Phone::new("a"))]
    ).eval(&[], 0, 0, &mut Choices::default(), Direction::LTR).unwrap());

    assert!(!Cond::new(
        CondType::Equality,
        vec![RuleToken::Phone(Phone::new("a"))],
        vec![RuleToken::Phone(Phone::new("b"))]
    ).eval(&[], 0, 0, &mut Choices::default(), Direction::LTR).unwrap());

    assert!(!Cond::new(
        CondType::Equality,
        vec![RuleToken::Phone(Phone::new("a a"))],
        vec![RuleToken::Phone(Phone::new("a")), RuleToken::Phone(Phone::new("a"))]
    ).eval(&[], 0, 0, &mut Choices::default(), Direction::LTR).unwrap());
}

#[test]
fn and() {
    let mut cond = Cond::new(
        CondType::MatchInput,
        vec![RuleToken::Phone(Phone::new("b"))],
        vec![RuleToken::Phone(Phone::new("d"))],
    );

    let phones_1 = &[
        Phone::new("a"),
        Phone::new("b"),
        Phone::new("c"),
        Phone::new("d"),
        Phone::new("e"),
    ];

    let phones_2 = &[
        Phone::new("a"),
        Phone::new("b"),
        Phone::new("c"),
        Phone::new("d"),
        Phone::new("f"),
    ];

    assert!(cond.eval(phones_1, 2, 1, &mut Choices::default(), Direction::LTR).unwrap());

    assert!(cond.eval(phones_2, 2, 1, &mut Choices::default(), Direction::LTR).unwrap());

    cond.set_and(Cond::new(
        CondType::MatchInput,
        vec![RuleToken::Phone(Phone::new("a")), RuleToken::Any { id: None }],
        vec![RuleToken::Any { id: None }, RuleToken::Phone(Phone::new("e"))],
    ));

    assert!(cond.eval(phones_1, 2, 1, &mut Choices::default(), Direction::LTR).unwrap());

    assert!(!cond.eval(phones_2, 2, 1, &mut Choices::default(), Direction::LTR).unwrap());
}