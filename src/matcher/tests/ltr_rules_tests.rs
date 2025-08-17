use crate::{matcher::{choices::Choices, match_state::MatchState, rule_pattern::RulePattern, Phones}, phones::Phone, rules::{conditions::Cond, tokens::RuleToken}, tokens::Direction};

#[test]
fn matches_phones() {
    let choices = Choices::default();
    let default_conds = [Cond::default()];

    let mut rules_pattern = RulePattern::new(
        &[RuleToken::Phone(Phone::Symbol("a"))],
        &default_conds,
        &[]
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(rules_pattern.next_match(&mut match_phones, &choices).is_some());
    let default_conds = [Cond::default()];

    let mut rules_pattern = RulePattern::new(
        &[
            RuleToken::Phone(Phone::Symbol("a")),
            RuleToken::Phone(Phone::Symbol("b")),
            RuleToken::Phone(Phone::Symbol("c")),
        ],
        &default_conds,
        &[]
    );
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")], 0, Direction::Ltr);

    assert!(rules_pattern.next_match(&mut match_phones, &choices).is_some());
}

// todo: more tests
// todo: conds, anti-conds, &, &!, with gaps