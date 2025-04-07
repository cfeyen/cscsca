use super::*;

#[test]
fn match_none() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[], &[], &mut Choices::default()));
}

#[test]
fn match_one_phone() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[RuleToken::Phone(Phone::Symbol("a"))], &[Phone::Symbol("a")], &mut Choices::default()));
}

#[test]
fn fail_match_one_phone() {
    assert_eq!(Ok(false), tokens_match_phones_from_right(&[RuleToken::Phone(Phone::Symbol("a"))], &[Phone::Symbol("b")], &mut Choices::default()));
}

#[test]
fn fail_match_one_token_to_no_phones() {
    assert_eq!(Ok(false), tokens_match_phones_from_right(&[RuleToken::Phone(Phone::Symbol("a"))], &[], &mut Choices::default()));
}

#[test]
fn match_three_phones() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::Phone(Phone::Symbol("a")),
        RuleToken::Phone(Phone::Symbol("b")),
        RuleToken::Phone(Phone::Symbol("c")),
    ], &[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")], &mut Choices::default()));
}

#[test]
fn fail_match_three_tokens_to_two_phones() {
    assert_eq!(Ok(false), tokens_match_phones_from_right(&[
        RuleToken::Phone(Phone::Symbol("a")),
        RuleToken::Phone(Phone::Symbol("b")),
        RuleToken::Phone(Phone::Symbol("c")),
    ], &[Phone::Symbol("a"), Phone::Symbol("b")], &mut Choices::default()));
}

#[test]
fn match_two_tokens_to_three_phones() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::Phone(Phone::Symbol("b")),
        RuleToken::Phone(Phone::Symbol("c")),
    ], &[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")], &mut Choices::default()));
}

#[test]
fn match_two_tokens_to_three_phones_from_late_start() {
    assert_eq!(Ok(true), tokens_match_phones(&[
        RuleToken::Phone(Phone::Symbol("b")),
        RuleToken::Phone(Phone::Symbol("c")),
    ], &[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")], 0, &mut 1, &mut Choices::default(), Direction::Rtl));
}

#[test]
fn match_option_to_nothing() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::OptionalScope { id: None, content: vec![
            RuleToken::Phone(Phone::Symbol("a"))
        ] },
    ], &[], &mut Choices::default()));
}

#[test]
fn match_option_to_phone() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::OptionalScope { id: None, content: vec![
            RuleToken::Phone(Phone::Symbol("a"))
        ] },
    ], &[Phone::Symbol("a")], &mut Choices::default()));
}

#[test]
fn match_option_phone_to_phone_phone() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::OptionalScope { id: None, content: vec![
            RuleToken::Phone(Phone::Symbol("a"))
        ] },
        RuleToken::Phone(Phone::Symbol("b")),
    ], &[Phone::Symbol("a"), Phone::Symbol("b")], &mut Choices::default()));
}

#[test]
fn match_option_to_phone_and_check_mapping() {
    let mut choices = Choices::default();
    let id = ScopeId::Name("label");

    let scope = [RuleToken::OptionalScope { id: Some(id.clone()), content: vec![
        RuleToken::Phone(Phone::Symbol("a"))
    ] }];

    _ = tokens_match_phones_from_right(&scope, &[Phone::Symbol("a")], &mut choices);

    assert_eq!(Some(&true), choices.optional_choices.get(&id))
}

#[test]
fn fail_match_same_labeled_optional_scopes_where_the_first_could_match_but_the_second_could_not() {
    let mut choices = Choices::default();
    let id = ScopeId::Name("label");

    let scope_1 = RuleToken::OptionalScope { id: Some(id.clone()), content: vec![
        RuleToken::Phone(Phone::Symbol("a"))
    ] };

    let scope_2 = RuleToken::OptionalScope { id: Some(id.clone()), content: vec![
        RuleToken::Phone(Phone::Symbol("c"))
    ] };

    let phone = RuleToken::Phone(Phone::Symbol("b"));

    let tokens = [scope_1, phone, scope_2];

    assert_eq!(Ok(false), tokens_match_phones_from_right(&tokens, &[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("d")], &mut choices));

    assert_eq!(Some(&false), choices.optional_choices.get(&id))
}

#[test]
fn fail_match_unlabeled_any_to_nothing() {
    assert_eq!(Ok(false), tokens_match_phones_from_right(&[
        RuleToken::Any { id: None },
    ], &[], &mut Choices::default()))
}

#[test]
fn fail_match_labeled_any_to_nothing() {
    assert_eq!(Ok(false), tokens_match_phones_from_right(&[
        RuleToken::Any { id: Some(ScopeId::Name("label")) },
    ], &[], &mut Choices::default()))
}

#[test]
fn match_unlabeled_any_to_phone() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::Any { id: None },
    ], &[Phone::Symbol("a")], &mut Choices::default()));

    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::Any { id: None },
    ], &[Phone::Symbol("b")], &mut Choices::default()));

    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::Any { id: None },
    ], &[Phone::Symbol("c")], &mut Choices::default()));
}

#[test]
fn match_labeled_any_to_phone() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::Any { id: Some(ScopeId::Name("label")) },
    ], &[Phone::Symbol("a")], &mut Choices::default()));

    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::Any { id: Some(ScopeId::Name("label")) },
    ], &[Phone::Symbol("b")], &mut Choices::default()));

    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::Any { id: Some(ScopeId::Name("label")) },
    ], &[Phone::Symbol("c")], &mut Choices::default()));
}

#[test]
fn match_pair_of_labeled_any_to_two_same_phones() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::Any { id: Some(ScopeId::Name("label")) },
        RuleToken::Any { id: Some(ScopeId::Name("label")) },
    ], &[Phone::Symbol("a"), Phone::Symbol("a")], &mut Choices::default()));
}

#[test]
fn fail_match_pair_of_labeled_any_to_two_different_phones() {
    assert_eq!(Ok(false), tokens_match_phones_from_right(&[
        RuleToken::Any { id: Some(ScopeId::Name("label")) },
        RuleToken::Any { id: Some(ScopeId::Name("label")) },
    ], &[Phone::Symbol("a"), Phone::Symbol("b")], &mut Choices::default()));
}

#[test]
fn fail_match_unlabeled_any_to_bound() {
    assert_eq!(Ok(false), tokens_match_phones_from_right(&[
        RuleToken::Any { id: None },
    ], &[Phone::Bound], &mut Choices::default()))
}

#[test]
fn fail_match_gap() {
    assert_eq!(Ok(false), tokens_match_phones_from_right(&[
        RuleToken::Phone(Phone::Symbol("a")),
        RuleToken::Gap { id: None },
        RuleToken::Phone(Phone::Symbol("b")),
    ], &[Phone::Symbol("a"), Phone::Bound, Phone::Symbol("b")], &mut Choices::default()))
}

#[test]
fn match_gap() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::Phone(Phone::Symbol("a")),
        RuleToken::Gap { id: None },
        RuleToken::Phone(Phone::Symbol("b")),
    ], &[Phone::Symbol("a"), Phone::Symbol("c"), Phone::Symbol("d"), Phone::Symbol("b")], &mut Choices::default()))
}

#[test]
fn match_zero_gap() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::Phone(Phone::Symbol("a")),
        RuleToken::Gap { id: None },
        RuleToken::Phone(Phone::Symbol("b")),
    ], &[Phone::Symbol("a"), Phone::Symbol("b")], &mut Choices::default()))
}

#[test]
fn match_shorter_labeled_gap() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::Phone(Phone::Symbol("a")),
        RuleToken::Gap { id: Some("label") },
        RuleToken::Phone(Phone::Symbol("b")),
        RuleToken::Gap { id: Some("label") },
        RuleToken::Phone(Phone::Symbol("c")),
    ], &[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("d"), Phone::Symbol("c")], &mut Choices::default()))
}

#[test]
fn match_equal_labeled_gap() {
    assert_eq!(Ok(true), tokens_match_phones_from_left(&[
        RuleToken::Phone(Phone::Symbol("a")),
        RuleToken::Gap { id: Some("label") },
        RuleToken::Phone(Phone::Symbol("b")),
        RuleToken::Gap { id: Some("label") },
        RuleToken::Phone(Phone::Symbol("c")),
    ], &[Phone::Symbol("a"), Phone::Symbol("d"), Phone::Symbol("b"), Phone::Symbol("d"), Phone::Symbol("c")], &mut Choices::default()))
}

#[test]
fn fail_match_longer_labeled_gap() {
    assert_eq!(Ok(false), tokens_match_phones_from_right(&[
        RuleToken::Phone(Phone::Symbol("a")),
        RuleToken::Gap { id: Some("label") },
        RuleToken::Phone(Phone::Symbol("b")),
        RuleToken::Gap { id: Some("label") },
        RuleToken::Phone(Phone::Symbol("c")),
    ], &[Phone::Symbol("a"), Phone::Symbol("d"), Phone::Symbol("d"), Phone::Symbol("b"), Phone::Symbol("d"), Phone::Symbol("c")], &mut Choices::default()))
}

#[test]
fn match_first_in_selection() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::SelectionScope { id: None, options: vec![
            vec![RuleToken::Phone(Phone::Symbol("a"))],
            vec![RuleToken::Phone(Phone::Symbol("b"))],
            vec![RuleToken::Phone(Phone::Symbol("c"))],
        ] },
    ], &[Phone::Symbol("a")], &mut Choices::default()))
}

#[test]
fn match_third_in_selection() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::SelectionScope { id: None, options: vec![
            vec![RuleToken::Phone(Phone::Symbol("a"))],
            vec![RuleToken::Phone(Phone::Symbol("b"))],
            vec![RuleToken::Phone(Phone::Symbol("c"))],
        ] },
    ], &[Phone::Symbol("c")], &mut Choices::default()))
}

#[test]
fn match_same_labeled_selection() {
    assert_eq!(Ok(true), tokens_match_phones_from_right(&[
        RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
            vec![RuleToken::Phone(Phone::Symbol("a"))],
            vec![RuleToken::Phone(Phone::Symbol("b"))],
            vec![RuleToken::Phone(Phone::Symbol("c"))],
        ] },
        RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
            vec![RuleToken::Phone(Phone::Symbol("d"))],
            vec![RuleToken::Phone(Phone::Symbol("e"))],
            vec![RuleToken::Phone(Phone::Symbol("f"))],
        ] },
    ], &[Phone::Symbol("a"), Phone::Symbol("d")], &mut Choices::default()))
}

#[test]
fn fail_match_selection() {
    assert_eq!(Ok(false), tokens_match_phones_from_left(&[
        RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
            vec![RuleToken::Phone(Phone::Symbol("a"))],
            vec![RuleToken::Phone(Phone::Symbol("b"))],
            vec![RuleToken::Phone(Phone::Symbol("c"))],
        ] },
    ], &[Phone::Symbol("d"), Phone::Symbol("e"), Phone::Symbol("f")], &mut Choices::default()))
}


#[test]
fn fail_match_same_labeled_selection() {
    assert_eq!(Ok(false), tokens_match_phones_from_right(&[
        RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
            vec![RuleToken::Phone(Phone::Symbol("a"))],
            vec![RuleToken::Phone(Phone::Symbol("b"))],
            vec![RuleToken::Phone(Phone::Symbol("c"))],
        ] },
        RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
            vec![RuleToken::Phone(Phone::Symbol("d"))],
            vec![RuleToken::Phone(Phone::Symbol("e"))],
            vec![RuleToken::Phone(Phone::Symbol("f"))],
        ] },
    ], &[Phone::Symbol("a"), Phone::Symbol("e")], &mut Choices::default()))
}

#[test]
fn match_different_labeled_selection() {
    let mut choices = Choices::default();

    let first_scope = RuleToken::SelectionScope { id: Some(ScopeId::Name("label_1")), options: vec![
        vec![RuleToken::Phone(Phone::Symbol("a"))],
        vec![RuleToken::Phone(Phone::Symbol("b"))],
        vec![RuleToken::Phone(Phone::Symbol("c"))],
    ] };

    let second_scope = RuleToken::SelectionScope { id: Some(ScopeId::Name("label_2")), options: vec![
        vec![RuleToken::Phone(Phone::Symbol("d"))],
        vec![RuleToken::Phone(Phone::Symbol("e"))],
        vec![RuleToken::Phone(Phone::Symbol("f"))],
    ] };

    let tokens = [first_scope, second_scope];

    assert_eq!(Ok(true), tokens_match_phones_from_right(&tokens, &[Phone::Symbol("a"), Phone::Symbol("e")], &mut choices));

    assert_eq!(choices.selection_choices.get(&ScopeId::Name("label_1")), Some(&0));

    assert_eq!(choices.selection_choices.get(&ScopeId::Name("label_2")), Some(&1));
}