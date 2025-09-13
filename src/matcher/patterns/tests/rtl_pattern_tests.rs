use crate::{matcher::{choices::Choices, match_state::{MatchState, UnitState}, patterns::{check_box::CheckBox, list::PatternList, Pattern}, phones::Phones}, phones::Phone, rules::tokens::ScopeId, tokens::Direction};

#[test]
fn single_phone() {
    let choices = Choices::default();

    let mut phone = Phone::Symbol("a");
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(UnitState::matches(&mut phone, &mut match_phones, &choices).is_some());

    let phone_box = CheckBox::new(phone);
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);
    
    assert!(phone_box.matches(&mut match_phones, &choices).is_some());

    let mut phone = Phone::Bound;
    let mut match_phones = Phones::new(&[Phone::Bound], 0, Direction::Rtl);

    assert!(UnitState::matches(&mut phone, &mut match_phones, &choices).is_some());

    let phone_box = CheckBox::new(phone);
    let mut match_phones = Phones::new(&[Phone::Bound], 0, Direction::Rtl);
    
    assert!(phone_box.matches(&mut match_phones, &choices).is_some());

    let mut phone = Phone::Symbol("a");
    let mut match_phones = Phones::new(&[Phone::Bound], 0, Direction::Rtl);

    assert!(UnitState::matches(&mut phone, &mut match_phones, &choices).is_none());

    let phone_box = CheckBox::new(phone);
    let mut match_phones = Phones::new(&[Phone::Bound], 0, Direction::Rtl);
    
    assert!(phone_box.matches(&mut match_phones, &choices).is_none());

    let mut phone = Phone::Bound;
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(UnitState::matches(&mut phone, &mut match_phones, &choices).is_none());

    let phone_box = CheckBox::new(phone);
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);
    
    assert!(phone_box.matches(&mut match_phones, &choices).is_none());

    let mut phone = Phone::Symbol("a");
    let mut match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Rtl);

    assert!(UnitState::matches(&mut phone, &mut match_phones, &choices).is_none());

    let phone_box = CheckBox::new(phone);
    let mut match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Rtl);
    
    assert!(phone_box.matches(&mut match_phones, &choices).is_none());
}

#[test]
fn multiple_phones() {
    let choices = Choices::default();

    let phones = vec![
        Pattern::new_phone(Phone::Symbol("a")), 
        Pattern::new_phone(Phone::Symbol("b")), 
        Pattern::new_phone(Phone::Symbol("c")),
    ];
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")], 2, Direction::Rtl);

    assert!(PatternList::new(phones).matches(&mut match_phones, &choices).is_some());

    let phones = vec![
        Pattern::new_phone(Phone::Symbol("a")), 
        Pattern::new_phone(Phone::Symbol("b")), 
        Pattern::new_phone(Phone::Symbol("c")),
    ];

    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("d")], 2, Direction::Rtl);

    assert!(PatternList::new(phones).matches(&mut match_phones, &choices).is_none());

    let phones = vec![
        Pattern::new_phone(Phone::Symbol("a")), 
        Pattern::new_phone(Phone::Symbol("b")), 
        Pattern::new_phone(Phone::Symbol("c")),
    ];

    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 1, Direction::Rtl);

    assert!(PatternList::new(phones).matches(&mut match_phones, &choices).is_none());
}

#[test]
fn empty_list() {
let choices = Choices::default();

    let patterns = PatternList::new(Vec::new());
    let mut match_phones = Phones::new(&[], 0, Direction::Rtl);

    assert!(patterns.matches(&mut match_phones, &choices).is_some());
}

#[test]
fn single_non_bound() {
    let choices = Choices::default();

    let phone = Pattern::new_any(None);
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(phone.matches(&mut match_phones, &choices).is_some());

    let phone = Pattern::new_any(None);
    let mut match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Rtl);

    assert!(phone.matches(&mut match_phones, &choices).is_some());

    let phone = Pattern::new_any(None);
    let mut match_phones = Phones::new(&[Phone::Bound], 0, Direction::Rtl);

    assert!(phone.matches(&mut match_phones, &choices).is_none());
}

#[test]
fn agreeing_non_bounds() {
    let mut choices = Choices::default();

    let label = ScopeId::Name("label");

    let patterns = vec![Pattern::new_any(Some(&label)), Pattern::new_any(Some(&label))];
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("a")], 1, Direction::Rtl);

    let Some(new_choices) = PatternList::new(patterns).matches(&mut match_phones, &choices) else {
        panic!("agreeing bounds did not patch");
    };

    choices.take_owned(new_choices);

    assert_eq!(choices.any.get(&label), Some(&Phone::Symbol("a")));

    let choices = Choices::default();

    let label = ScopeId::Name("label");

    let patterns = vec![Pattern::new_any(Some(&label)), Pattern::new_any(Some(&label))];
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 1, Direction::Rtl);

    assert!(PatternList::new(patterns).matches(&mut match_phones, &choices).is_none());
}

#[test]
fn unbounded_gap() {
    let choices = Choices::default();

    let pattern = Pattern::new_gap(None);
    let mut match_phones = Phones::new(&[], 0, Direction::Rtl);

    assert!(pattern.matches(&mut match_phones, &choices).is_some());
}

#[test]
fn bounded_gap() {
    let choices = Choices::default();

    let mut patterns = PatternList::new(vec![
        Pattern::new_phone(Phone::Symbol("a")),
        Pattern::new_gap(None),
        Pattern::new_phone(Phone::Symbol("b")),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 1, Direction::Rtl);

    assert!(patterns.next_match(&mut match_phones, &choices).is_some());

    let mut patterns = PatternList::new(vec![
        Pattern::new_phone(Phone::Symbol("a")),
        Pattern::new_gap(None),
        Pattern::new_phone(Phone::Symbol("b")),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("-"), Phone::Symbol("b")], 2, Direction::Rtl);

    assert!(patterns.next_match(&mut match_phones, &choices).is_some());

    let mut patterns = PatternList::new(vec![
        Pattern::new_phone(Phone::Symbol("a")),
        Pattern::new_gap(None),
        Pattern::new_phone(Phone::Symbol("b")),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("-"), Phone::Symbol("-"), Phone::Symbol("-"), Phone::Symbol("b")], 4, Direction::Rtl);

    assert!(patterns.next_match(&mut match_phones, &choices).is_some());

    let mut patterns = PatternList::new(vec![
        Pattern::new_phone(Phone::Symbol("a")),
        Pattern::new_gap(None),
        Pattern::new_phone(Phone::Symbol("b")),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Bound, Phone::Symbol("b")], 2, Direction::Rtl);

    assert!(patterns.next_match(&mut match_phones, &choices).is_none());
}

#[test]
fn agreeing_gaps() {
    let choices = Choices::default();

    let label = "label";

    let mut patterns = PatternList::new(vec![
        Pattern::new_phone(Phone::Symbol("a")),
        Pattern::new_gap(Some(label)),
        Pattern::new_phone(Phone::Symbol("b")),
        Pattern::new_gap(Some(label)),
        Pattern::new_phone(Phone::Symbol("c")),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("-"), Phone::Symbol("b"), Phone::Symbol("c"),], 3, Direction::Rtl);

    assert!(patterns.next_match(&mut match_phones, &choices).is_none());

    let mut patterns = PatternList::new(vec![
        Pattern::new_phone(Phone::Symbol("a")),
        Pattern::new_gap(Some(label)),
        Pattern::new_phone(Phone::Symbol("b")),
        Pattern::new_gap(Some(label)),
        Pattern::new_phone(Phone::Symbol("c")),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("-"), Phone::Symbol("b"), Phone::Symbol("-"), Phone::Symbol("c"),], 4, Direction::Rtl);

    assert!(patterns.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(patterns.len(), 5);

    let mut patterns = PatternList::new(vec![
        Pattern::new_phone(Phone::Symbol("a")),
        Pattern::new_gap(Some(label)),
        Pattern::new_phone(Phone::Symbol("b")),
        Pattern::new_gap(Some(label)),
        Pattern::new_phone(Phone::Symbol("c")),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("-"), Phone::Symbol("b"), Phone::Symbol("-"), Phone::Symbol("-"), Phone::Symbol("c"),], 5, Direction::Rtl);

    assert!(patterns.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(patterns.len(), 6);
}

#[test]
fn optional() {
    let choices = Choices::default();

    let mut pattern = Pattern::new_optional(vec![
        Pattern::new_phone(Phone::Symbol("a")),
        Pattern::new_phone(Phone::Symbol("b")),
        Pattern::new_phone(Phone::Symbol("c")),
    ], None);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")], 2, Direction::Rtl);

    assert!(pattern.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(pattern.len(), 3);

    let mut pattern = Pattern::new_optional(vec![
        Pattern::new_phone(Phone::Symbol("a")),
        Pattern::new_phone(Phone::Symbol("b")),
        Pattern::new_phone(Phone::Symbol("c")),
    ], None);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("d")], 2, Direction::Rtl);

    assert!(pattern.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(pattern.len(), 0);
}

#[test]
fn agreeing_optionals() {
    let choices = Choices::default();

    let label = ScopeId::Name("label");

    let mut patterns = PatternList::new(vec![
        Pattern::new_optional(vec![
            Pattern::new_phone(Phone::Symbol("a")),
        ], Some(&label)),
        Pattern::new_optional(vec![
            Pattern::new_phone(Phone::Symbol("b")),
        ], Some(&label)),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("a"), Phone::Symbol("b")], 1, Direction::Rtl);

    assert!(patterns.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(patterns.len(), 2);

    let mut patterns = PatternList::new(vec![
        Pattern::new_optional(vec![
            Pattern::new_phone(Phone::Symbol("a")),
        ], Some(&label)),
        Pattern::new_optional(vec![
            Pattern::new_phone(Phone::Symbol("b")),
        ], Some(&label)),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("b")], 1, Direction::Rtl);

    assert!(patterns.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(patterns.len(), 0);
}

#[test]
fn selection() {
    let choices = Choices::default();

    let mut pattern = Pattern::new_selection(vec![
        vec![Pattern::new_phone(Phone::Symbol("a"))],
        vec![Pattern::new_phone(Phone::Symbol("b"))],
        vec![Pattern::new_phone(Phone::Symbol("c"))],
    ], None);
    let mut match_phones = Phones::new(&[Phone::Symbol("a")], 0, Direction::Rtl);

    assert!(pattern.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(pattern.len(), 1);

    let mut pattern = Pattern::new_selection(vec![
        vec![Pattern::new_phone(Phone::Symbol("a"))],
        vec![Pattern::new_phone(Phone::Symbol("b"))],
        vec![Pattern::new_phone(Phone::Symbol("c"))],
    ], None);
    let mut match_phones = Phones::new(&[Phone::Symbol("b")], 0, Direction::Rtl);

    assert!(pattern.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(pattern.len(), 1);

    let mut pattern = Pattern::new_selection(vec![
        vec![Pattern::new_phone(Phone::Symbol("a"))],
        vec![Pattern::new_phone(Phone::Symbol("b"))],
        vec![Pattern::new_phone(Phone::Symbol("c"))],
    ], None);
    let mut match_phones = Phones::new(&[Phone::Symbol("c")], 0, Direction::Rtl);

    assert!(pattern.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(pattern.len(), 1);
}

#[test]
fn agreeing_selection() {
    let choices = Choices::default();

    let label = ScopeId::Name("label");

    let mut patterns = PatternList::new(vec![
        Pattern::new_selection(vec![
            vec![Pattern::new_phone(Phone::Symbol("a"))],
            vec![Pattern::new_phone(Phone::Symbol("b"))],
            vec![Pattern::new_phone(Phone::Symbol("c"))],
        ], Some(&label)),
        Pattern::new_selection(vec![
            vec![Pattern::new_phone(Phone::Symbol("d"))],
            vec![Pattern::new_phone(Phone::Symbol("e"))],
            vec![Pattern::new_phone(Phone::Symbol("f"))],
        ], Some(&label)),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("e")], 1, Direction::Rtl);

    assert!(patterns.next_match(&mut match_phones, &choices).is_some());
    assert_eq!(patterns.len(), 2);

    let mut patterns = PatternList::new(vec![
        Pattern::new_selection(vec![
            vec![Pattern::new_phone(Phone::Symbol("a"))],
            vec![Pattern::new_phone(Phone::Symbol("b"))],
            vec![Pattern::new_phone(Phone::Symbol("c"))],
        ], Some(&label)),
        Pattern::new_selection(vec![
            vec![Pattern::new_phone(Phone::Symbol("d"))],
            vec![Pattern::new_phone(Phone::Symbol("e"))],
            vec![Pattern::new_phone(Phone::Symbol("f"))],
        ], Some(&label)),
    ]);
    let mut match_phones = Phones::new(&[Phone::Symbol("b"), Phone::Symbol("d")], 1, Direction::Rtl);

    assert!(patterns.next_match(&mut match_phones, &choices).is_none());
}