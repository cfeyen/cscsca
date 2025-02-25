use super::*;

#[test]
fn phone() {
    assert!(!has_empty_form(&[RuleToken::Phone(Phone::new("a"))]));
}

#[test]
fn optional() {
    assert!(has_empty_form(&[RuleToken::OptionalScope { id: None, content: vec![
        RuleToken::Phone(Phone::new("a"))
    ] }]));
}

#[test]
fn full_selection() {
    assert!(!has_empty_form(&[RuleToken::SelectionScope { id: None, options: vec![
        vec![RuleToken::Phone(Phone::new("a"))],
        vec![RuleToken::Phone(Phone::new("b"))],
        vec![RuleToken::Phone(Phone::new("c"))],
    ] }]));
}

#[test]
fn non_full_selection() {
    assert!(has_empty_form(&[RuleToken::SelectionScope { id: None, options: vec![
        vec![RuleToken::Phone(Phone::new("a"))],
        vec![RuleToken::Phone(Phone::new("b"))],
        vec![],
    ] }]));
}

#[test]
fn option_in_selection() {
    assert!(has_empty_form(&[RuleToken::SelectionScope { id: None, options: vec![
        vec![RuleToken::Phone(Phone::new("a"))],
        vec![RuleToken::Phone(Phone::new("b"))],
        vec![RuleToken::OptionalScope { id: None, content: vec![RuleToken::Any { id: None }] }],
    ] }]));
}

#[test]
fn option_in_selection_in_selection() {
    assert!(has_empty_form(&[RuleToken::SelectionScope { id: None, options: vec![
        vec![RuleToken::Phone(Phone::new("a"))],
        vec![RuleToken::Phone(Phone::new("b"))],
        vec![RuleToken::SelectionScope { id: None, options: vec![
            vec![RuleToken::Phone(Phone::new("a"))],
            vec![RuleToken::Phone(Phone::new("b"))],
            vec![RuleToken::OptionalScope { id: None, content: vec![RuleToken::Any { id: None }] }],
        ] }],
    ] }]));
}

#[test]
fn any() {
    assert!(!has_empty_form(&[RuleToken::Any { id: None }]));
}