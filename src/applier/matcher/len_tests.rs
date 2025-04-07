use crate::rules::sound_change_rule::LabelType;

use super::*;

#[test]
fn empty() {
    assert_eq!(Ok(0), match_len(&[], &Choices::default()));
}

#[test]
fn phone() {
    assert_eq!(Ok(1), match_len(&[
        RuleToken::Phone(Phone::Symbol("a")),
    ], &Choices::default()));
}

#[test]
fn three_phoens() {
    assert_eq!(Ok(3), match_len(&[
        RuleToken::Phone(Phone::Symbol("a")),
        RuleToken::Phone(Phone::Symbol("b")),
        RuleToken::Phone(Phone::Symbol("c")),
    ], &Choices::default()));
}

#[test]
fn gap() {
    assert_eq!(Err(MatchError::CannotCheckLenOfGap), match_len(&[
        RuleToken::Gap { id: None },
    ], &Choices::default()));
}

#[test]
fn unlabeled_optional() {
    let scope = RuleToken::OptionalScope { id: None, content: vec![
        RuleToken::Phone(Phone::Symbol("a")),
        RuleToken::Phone(Phone::Symbol("b")),
    ] };

    assert_eq!(Err(MatchError::UnlabeledScope(&scope.clone())), match_len(&[
        scope,
    ], &Choices::default()));
}

#[test]
fn inserted_optional() {
    let mut choices = Choices::default();
    choices.optional_choices = HashMap::from_iter([(&ScopeId::Name("label"), true)]);

    let scope = RuleToken::OptionalScope { id: Some(ScopeId::Name("label")), content: vec![
        RuleToken::Phone(Phone::Symbol("a")),
        RuleToken::Phone(Phone::Symbol("b")),
    ] };

    assert_eq!(Ok(2), match_len(&[scope], &choices));
}

#[test]
fn non_inserted_optional() {
    let mut choices = Choices::default();
    choices.optional_choices = HashMap::from_iter([(&ScopeId::Name("label"), false)]);

    let scope = RuleToken::OptionalScope { id: Some(ScopeId::Name("label")), content: vec![
        RuleToken::Phone(Phone::Symbol("a")),
        RuleToken::Phone(Phone::Symbol("b")),
    ] };

    assert_eq!(Ok(0), match_len(&[scope], &choices));
}

#[test]
fn unlabeled_selection() {
    let scope = RuleToken::SelectionScope { id: None, options: vec![
        vec![RuleToken::Phone(Phone::Symbol("a"))],
        vec![RuleToken::Phone(Phone::Symbol("b")), RuleToken::Phone(Phone::Symbol("c"))],
    ] };

    assert_eq!(Err(MatchError::UnlabeledScope(&scope.clone())), match_len(&[
        scope,
    ], &Choices::default()));
}

#[test]
fn first_in_selection() {
    let mut choices = Choices::default();
    choices.selection_choices = HashMap::from_iter([(&ScopeId::Name("label"), 0)]);

    let scope = RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
        vec![RuleToken::Phone(Phone::Symbol("a"))],
        vec![RuleToken::Phone(Phone::Symbol("b")), RuleToken::Phone(Phone::Symbol("c"))],
    ] };

    assert_eq!(Ok(1), match_len(&[scope], &choices));
}

#[test]
fn second_in_selection() {
    let mut choices = Choices::default();
    choices.selection_choices = HashMap::from_iter([(&ScopeId::Name("label"), 1)]);

    let scope = RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
        vec![RuleToken::Phone(Phone::Symbol("a"))],
        vec![RuleToken::Phone(Phone::Symbol("b")), RuleToken::Phone(Phone::Symbol("c"))],
    ] };

    assert_eq!(Ok(2), match_len(&[scope], &choices));
}

#[test]
fn any_equal_phone() {


    assert_eq!(match_len(&[
        RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) },
    ], &Choices::default()), match_len(&[
        RuleToken::Phone(Phone::Symbol("a")),
    ], &Choices::default()));
}