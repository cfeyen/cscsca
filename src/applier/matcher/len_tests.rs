use crate::rules::sound_change_rule::LabelType;

use super::*;

#[test]
fn empty() {
    assert_eq!(Ok(0), match_len(&vec![], &Choices::default()));
}

#[test]
fn phone() {
    assert_eq!(Ok(1), match_len(&vec![
        RuleToken::Phone(Phone::new("a")),
    ], &Choices::default()));
}

#[test]
fn three_phoens() {
    assert_eq!(Ok(3), match_len(&vec![
        RuleToken::Phone(Phone::new("a")),
        RuleToken::Phone(Phone::new("b")),
        RuleToken::Phone(Phone::new("c")),
    ], &Choices::default()));
}

#[test]
fn gap() {
    assert_eq!(Err(MatchError::CannotCheckLenOfGap), match_len(&vec![
        RuleToken::Gap { id: None },
    ], &Choices::default()));
}

#[test]
fn unlabeled_optional() {
    let scope = RuleToken::OptionalScope { id: None, content: vec![
        RuleToken::Phone(Phone::new("a")),
        RuleToken::Phone(Phone::new("b")),
    ] };

    assert_eq!(Err(MatchError::UnlabeledScope(&scope.clone())), match_len(&vec![
        scope,
    ], &Choices::default()));
}

#[test]
fn inserted_optional() {
    let mut choices = Choices::default();
    choices.optional_choices = HashMap::from_iter([(&ScopeId::Name("label"), true)].into_iter());

    let scope = RuleToken::OptionalScope { id: Some(ScopeId::Name("label")), content: vec![
        RuleToken::Phone(Phone::new("a")),
        RuleToken::Phone(Phone::new("b")),
    ] };

    assert_eq!(Ok(2), match_len(&vec![scope], &choices));
}

#[test]
fn non_inserted_optional() {
    let mut choices = Choices::default();
    choices.optional_choices = HashMap::from_iter([(&ScopeId::Name("label"), false)].into_iter());

    let scope = RuleToken::OptionalScope { id: Some(ScopeId::Name("label")), content: vec![
        RuleToken::Phone(Phone::new("a")),
        RuleToken::Phone(Phone::new("b")),
    ] };

    assert_eq!(Ok(0), match_len(&vec![scope], &choices));
}

#[test]
fn unlabeled_selection() {
    let scope = RuleToken::SelectionScope { id: None, options: vec![
        vec![RuleToken::Phone(Phone::new("a"))],
        vec![RuleToken::Phone(Phone::new("b")), RuleToken::Phone(Phone::new("c"))],
    ] };

    assert_eq!(Err(MatchError::UnlabeledScope(&scope.clone())), match_len(&vec![
        scope,
    ], &Choices::default()));
}

#[test]
fn first_in_selection() {
    let mut choices = Choices::default();
    choices.selection_choices = HashMap::from_iter([(&ScopeId::Name("label"), 0)].into_iter());

    let scope = RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
        vec![RuleToken::Phone(Phone::new("a"))],
        vec![RuleToken::Phone(Phone::new("b")), RuleToken::Phone(Phone::new("c"))],
    ] };

    assert_eq!(Ok(1), match_len(&vec![scope], &choices));
}

#[test]
fn second_in_selection() {
    let mut choices = Choices::default();
    choices.selection_choices = HashMap::from_iter([(&ScopeId::Name("label"), 1)].into_iter());

    let scope = RuleToken::SelectionScope { id: Some(ScopeId::Name("label")), options: vec![
        vec![RuleToken::Phone(Phone::new("a"))],
        vec![RuleToken::Phone(Phone::new("b")), RuleToken::Phone(Phone::new("c"))],
    ] };

    assert_eq!(Ok(2), match_len(&vec![scope], &choices));
}

#[test]
fn any_equal_phone() {


    assert_eq!(match_len(&vec![
        RuleToken::Any { id: Some(ScopeId::IOUnlabeled { id_num: 0, label_type: LabelType::Any, parent: None }) },
    ], &Choices::default()), match_len(&vec![
        RuleToken::Phone(Phone::new("a")),
    ], &Choices::default()));
}