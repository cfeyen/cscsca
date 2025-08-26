use crate::{matcher::{rule_pattern::RulePattern, Phones}, phones::Phone, rules::{conditions::{AndType, Cond, CondType}, tokens::RuleToken}, tokens::Direction};

#[test]
fn matches_phones() {
    let default_conds = [Cond::default()];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &[]
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_some());
    let default_conds = [Cond::default()];

    let mut rule_pattern = RulePattern::new(
        &[
            RuleToken::Phone(Phone::Symbol("a")),
            RuleToken::Phone(Phone::Symbol("b")),
            RuleToken::Phone(Phone::Symbol("c")),
        ],
        &default_conds,
        &[]
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_some());
}

#[test]
fn match_phone_with_cond() {
    let before_b = [Cond::new(CondType::Pattern, Vec::new(), vec![RuleToken::Phone(Phone::Symbol("b"))])];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &before_b,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &before_b,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &before_b,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());

    let after_b = [Cond::new(CondType::Pattern, vec![RuleToken::Phone(Phone::Symbol("b"))], Vec::new())];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &after_b,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &after_b,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &after_b,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());
}

#[test]
fn match_phone_with_anti_cond() {
    let default_conds = [Cond::default()];

    let before_b = [Cond::new(CondType::Pattern, Vec::new(), vec![RuleToken::Phone(Phone::Symbol("b"))])];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &before_b,
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &before_b,
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &before_b,
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());

    let after_b = [Cond::new(CondType::Pattern, vec![RuleToken::Phone(Phone::Symbol("b"))], Vec::new())];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &after_b,
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &after_b,
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &after_b,
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());
}

#[test]
fn and_cond() {
    let mut between_b_and_c = Cond::new(CondType::Pattern, vec![RuleToken::Phone(Phone::Symbol("b"))], Vec::new());
    let before_c = Cond::new(CondType::Pattern, Vec::new(), vec![RuleToken::Phone(Phone::Symbol("c"))]);

    between_b_and_c.set_and(AndType::And, before_c);

    let cond = [between_b_and_c];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a"), Phone::Symbol("c")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("c")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());
}

#[test]
fn and_not_cond() {
    let mut between_b_and_c = Cond::new(CondType::Pattern, vec![RuleToken::Phone(Phone::Symbol("b"))], Vec::new());
    let before_c = Cond::new(CondType::Pattern, Vec::new(), vec![RuleToken::Phone(Phone::Symbol("c"))]);

    between_b_and_c.set_and(AndType::AndNot, before_c);

    let cond = [between_b_and_c];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a"), Phone::Symbol("c")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("c")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_some());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    ).expect("pattern construction should be valid");
    let match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(match_phones).expect("next match should not error").is_none());
}

// todo: more tests
// todo: conds, anti-conds, &, &!, with gaps, non phone conds
// todo: test equality conds causing errors, succeeding, and failing