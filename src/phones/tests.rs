use super::*;

#[test]
fn text_matches_text () {
    assert!(Phone::Symbol("test").matches(&Phone::Symbol("test")));
    assert!(!Phone::Symbol("test").matches(&Phone::Symbol("not test")));
}

#[test]
fn escapes() {
    assert!(Phone::Symbol("\\@").matches(&Phone::Symbol("@")));
    assert!(!Phone::Symbol("@").matches(&Phone::Symbol("\\@")));
    assert!(Phone::Symbol("\\\\@").matches(&Phone::Symbol("\\@")));
    assert!(!Phone::Symbol("\\@").matches(&Phone::Symbol("\\@")));
    assert!(Phone::Symbol(&format!("\\{BOUND_STR}")).matches(&Phone::Bound));
    assert!(!Phone::Bound.matches(&Phone::Symbol(&format!("\\{BOUND_STR}"))));
}

#[test]
fn bound_matches_phone() {
    assert!(Phone::Symbol("\\ ").matches(&Phone::Bound));
    assert!(Phone::Symbol("\\ \\ ").matches(&Phone::Bound));
    assert!(!Phone::Bound.matches(&Phone::Symbol("\\ ")));
    assert!(Phone::Symbol("\\ ").matches(&Phone::Symbol(" ")));
}