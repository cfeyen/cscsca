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