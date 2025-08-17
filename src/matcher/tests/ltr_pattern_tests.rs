use crate::{matcher::{choices::Choices, match_state::MatchState, pattern::{Pattern, PatternList}, Phones}, phones::Phone, rules::tokens::ScopeId, tokens::Direction};

#[test]
fn single_phone() {
    let choices = Choices::default();

    let mut phone = Phone::Symbol("a");
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(MatchState::matches(&mut phone, &mut match_phones, &choices).is_some());

    let mut phone = Phone::Bound;
    let mut match_phones = Phones::new(&[Phone::Bound], 0, Direction::Ltr);

    assert!(MatchState::matches(&mut phone, &mut match_phones, &choices).is_some());

    let mut phone = Phone::Symbol("a");
    let mut match_phones = Phones::new(&[Phone::Bound], 0, Direction::Ltr);

    assert!(MatchState::matches(&mut phone, &mut match_phones, &choices).is_none());

    let mut phone = Phone::Bound;
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(MatchState::matches(&mut phone, &mut match_phones, &choices).is_none());

    let mut phone = Phone::Symbol("a");
    let mut match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(MatchState::matches(&mut phone, &mut match_phones, &choices).is_none());
}

#[test]
fn multiple_phones() {
    let choices = Choices::default();

    let phones = vec![
        Pattern::Phone(Phone::Symbol("a")), 
        Pattern::Phone(Phone::Symbol("b")), 
        Pattern::Phone(Phone::Symbol("c")),
    ];
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")], 0, Direction::Ltr);

    assert!(PatternList::new(phones).matches(&mut match_phones, &choices).is_some());

    let phones = vec![
        Pattern::Phone(Phone::Symbol("a")), 
        Pattern::Phone(Phone::Symbol("b")), 
        Pattern::Phone(Phone::Symbol("c")),
    ];

    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("d")], 0, Direction::Ltr);

    assert!(PatternList::new(phones).matches(&mut match_phones, &choices).is_none());

    let phones = vec![
        Pattern::Phone(Phone::Symbol("a")), 
        Pattern::Phone(Phone::Symbol("b")), 
        Pattern::Phone(Phone::Symbol("c")),
    ];

    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(PatternList::new(phones).matches(&mut match_phones, &choices).is_none());
}

#[test]
fn empty_list() {
let choices = Choices::default();

    let mut patterns = PatternList::new(Vec::new());
    let mut match_phones = Phones::new(&[], 0, Direction::Ltr);

    assert!(patterns.matches(&mut match_phones, &choices).is_some());
}

#[test]
fn single_non_bound() {
    let choices = Choices::default();

    let mut phone = Pattern::new_any(None);
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(phone.matches(&mut match_phones, &choices).is_some());

    let mut phone = Pattern::new_any(None);
    let mut match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(phone.matches(&mut match_phones, &choices).is_some());

    let mut phone = Pattern::new_any(None);
    let mut match_phones = Phones::new(&[Phone::Bound], 0, Direction::Ltr);

    assert!(phone.matches(&mut match_phones, &choices).is_none());
}

#[test]
fn agreeing_non_bounds() {
    let mut choices = Choices::default();

    let label = ScopeId::Name("label");

    let patterns = vec![Pattern::new_any(Some(&label)), Pattern::new_any(Some(&label))];
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("a")], 0, Direction::Ltr);

    let Some(new_choices) = PatternList::new(patterns).matches(&mut match_phones, &choices) else {
        panic!("agreeing bounds did not patch");
    };

    choices.take_owned(new_choices);

    assert_eq!(choices.any.get(&label), Some(&Phone::Symbol("a")));

    let choices = Choices::default();

    let label = ScopeId::Name("label");

    let patterns = vec![Pattern::new_any(Some(&label)), Pattern::new_any(Some(&label))];
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(PatternList::new(patterns).matches(&mut match_phones, &choices).is_none());
}

#[test]
fn unbounded_gap() {
    let choices = Choices::default();

    let mut pattern = Pattern::new_gap(None);
    let mut match_phones = Phones::new(&[], 0, Direction::Ltr);

    assert!(pattern.matches(&mut match_phones, &choices).is_some());
}

#[test]
fn bounded_gap() {
    let choices = Choices::default();

    let mut patterns = PatternList::new(vec![
        Pattern::Phone(Phone::Symbol("a")),
        Pattern::new_gap(None),
        Pattern::Phone(Phone::Symbol("b")),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(patterns.next_match(&mut match_phones, &choices).is_some());

    let mut patterns = PatternList::new(vec![
        Pattern::Phone(Phone::Symbol("a")),
        Pattern::new_gap(None),
        Pattern::Phone(Phone::Symbol("b")),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("-"), Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(patterns.next_match(&mut match_phones, &choices).is_some());

    let mut patterns = PatternList::new(vec![
        Pattern::Phone(Phone::Symbol("a")),
        Pattern::new_gap(None),
        Pattern::Phone(Phone::Symbol("b")),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("-"), Phone::Symbol("-"), Phone::Symbol("-"), Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(patterns.next_match(&mut match_phones, &choices).is_some());

    let mut patterns = PatternList::new(vec![
        Pattern::Phone(Phone::Symbol("a")),
        Pattern::new_gap(None),
        Pattern::Phone(Phone::Symbol("b")),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Bound, Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(patterns.next_match(&mut match_phones, &choices).is_none());
}

// todo: agreeing gaps

#[test]
fn optional() {
    let choices = Choices::default();

    let mut pattern = Pattern::new_optional(vec![
        Pattern::Phone(Phone::Symbol("a")),
        Pattern::Phone(Phone::Symbol("b")),
        Pattern::Phone(Phone::Symbol("c")),
    ], None);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")], 0, Direction::Ltr);

    assert!(pattern.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(pattern.len(), 3);

    let mut pattern = Pattern::new_optional(vec![
        Pattern::Phone(Phone::Symbol("a")),
        Pattern::Phone(Phone::Symbol("b")),
        Pattern::Phone(Phone::Symbol("c")),
    ], None);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("d")], 0, Direction::Ltr);

    assert!(pattern.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(pattern.len(), 0);
}

#[test]
fn agreeing_optionals() {
    let choices = Choices::default();

    let label = ScopeId::Name("label");

    let mut patterns = PatternList::new(vec![
        Pattern::new_optional(vec![
            Pattern::Phone(Phone::Symbol("a")),
        ], Some(&label)),
        Pattern::new_optional(vec![
            Pattern::Phone(Phone::Symbol("b")),
        ], Some(&label)),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(patterns.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(patterns.len(), 2);

    let mut patterns = PatternList::new(vec![
        Pattern::new_optional(vec![
            Pattern::Phone(Phone::Symbol("a")),
        ], Some(&label)),
        Pattern::new_optional(vec![
            Pattern::Phone(Phone::Symbol("b")),
        ], Some(&label)),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(patterns.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(patterns.len(), 0);
}

#[test]
fn selection() {
    let choices = Choices::default();

    let mut pattern = Pattern::new_selection(vec![
        vec![Pattern::Phone(Phone::Symbol("a"))],
        vec![Pattern::Phone(Phone::Symbol("b"))],
        vec![Pattern::Phone(Phone::Symbol("c"))],
    ], None);
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Ltr);

    assert!(pattern.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(pattern.len(), 1);

    let mut pattern = Pattern::new_selection(vec![
        vec![Pattern::Phone(Phone::Symbol("a"))],
        vec![Pattern::Phone(Phone::Symbol("b"))],
        vec![Pattern::Phone(Phone::Symbol("c"))],
    ], None);
    let mut match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Ltr);

    assert!(pattern.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(pattern.len(), 1);

    let mut pattern = Pattern::new_selection(vec![
        vec![Pattern::Phone(Phone::Symbol("a"))],
        vec![Pattern::Phone(Phone::Symbol("b"))],
        vec![Pattern::Phone(Phone::Symbol("c"))],
    ], None);
    let mut match_phones = Phones::new(&[Phone::Symbol("c")], 0, Direction::Ltr);

    assert!(pattern.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(pattern.len(), 1);
}


// todo: more tests