use crate::{matcher::{choices::Choices, match_state::MatchState, rule_pattern::RulePattern, Phones}, phones::Phone, rules::{conditions::{AndType, Cond, CondType}, tokens::RuleToken}, tokens::Direction};

#[test]
fn matches_phones() {
    let choices = Choices::default();
    let default_conds = [Cond::default()];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &[]
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_some());
    let default_conds = [Cond::default()];

    let mut rule_pattern = RulePattern::new(
        &[
            RuleToken::Phone(Phone::Symbol("a")),
            RuleToken::Phone(Phone::Symbol("b")),
            RuleToken::Phone(Phone::Symbol("c")),
        ],
        &default_conds,
        &[]
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_some());
}

#[test]
fn match_phone_with_cond() {
    let choices = Choices::default();

    let before_b = [Cond::new(CondType::Pattern, Vec::new(), vec![RuleToken::Phone(Phone::Symbol("b"))])];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &before_b,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_some());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &before_b,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &before_b,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());

    let after_b = [Cond::new(CondType::Pattern, vec![RuleToken::Phone(Phone::Symbol("b"))], Vec::new())];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &after_b,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_some());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &after_b,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &after_b,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("b")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());
}

#[test]
fn match_phone_with_anti_cond() {
    let choices = Choices::default();
    let default_conds = [Cond::default()];

    let before_b = [Cond::new(CondType::Pattern, Vec::new(), vec![RuleToken::Phone(Phone::Symbol("b"))])];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &before_b,
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &before_b,
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_some());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &before_b,
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());

    let after_b = [Cond::new(CondType::Pattern, vec![RuleToken::Phone(Phone::Symbol("b"))], Vec::new())];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &after_b,
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &after_b,
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_some());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &after_b,
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());
}

#[test]
fn and_cond() {
    let choices = Choices::default();

    let mut between_b_and_c = Cond::new(CondType::Pattern, vec![RuleToken::Phone(Phone::Symbol("b"))], Vec::new());
    let before_c = Cond::new(CondType::Pattern, Vec::new(), vec![RuleToken::Phone(Phone::Symbol("c"))]);

    between_b_and_c.set_and(AndType::And, before_c);

    let cond = [between_b_and_c];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a"), Phone::Symbol("c")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_some());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("c")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());
}

#[test]
fn and_not_cond() {
    let choices = Choices::default();

    let mut between_b_and_c = Cond::new(CondType::Pattern, vec![RuleToken::Phone(Phone::Symbol("b"))], Vec::new());
    let before_c = Cond::new(CondType::Pattern, Vec::new(), vec![RuleToken::Phone(Phone::Symbol("c"))]);

    between_b_and_c.set_and(AndType::AndNot, before_c);

    let cond = [between_b_and_c];

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a"), Phone::Symbol("c")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("c")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("a")], 1, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_some());

    let mut rule_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &cond,
        &[],
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rule_pattern.next_match(&mut match_phones, &choices).is_none());
}

// todo: more tests
// todo: conds, anti-conds, &, &!, with gaps, non phone conds