use crate::{apply, keywords::BOUND_CHAR};

use super::*;

#[test]
fn shifting_bounds() {
    assert_eq!(
        apply("a #c", "\\# >> b"),
        "a bc".to_string()
    )
}

#[test]
fn phone_list() {
    assert_eq!(
        build_phone_list("a b c"),
        vec![Phone::Symbol("a"), Phone::Bound, Phone::Symbol("b"), Phone::Bound, Phone::Symbol("c")]
    );

    assert_eq!(
        build_phone_list("ab c"),
        vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Bound, Phone::Symbol("c")]
    );

    assert_eq!(
        build_phone_list(&format!("a{BOUND_CHAR}b c")),
        vec![Phone::Symbol("a"), Phone::Symbol(&BOUND_CHAR.to_string()), Phone::Symbol("b"), Phone::Bound, Phone::Symbol("c")]
    );
}

#[test]
fn phone_list_to_str() {
    assert_eq!(
        phone_list_to_string(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")]),
        "abc".to_string()
    );

    assert_eq!(
        phone_list_to_string(&[Phone::Symbol("a"), Phone::Bound, Phone::Symbol("b"), Phone::Bound, Phone::Symbol("c")]),
        "a b c".to_string()
    );

    assert_eq!(
        phone_list_to_string(&[Phone::Symbol("a"), Phone::Bound, Phone::Symbol(&format!("{ESCAPE_CHAR}b")), Phone::Bound, Phone::Symbol("c")]),
        "a b c".to_string()
    );

    assert_eq!(
        phone_list_to_string(&[Phone::Symbol("a"), Phone::Bound, Phone::Symbol(&format!("{ESCAPE_CHAR}{ESCAPE_CHAR}b")), Phone::Bound, Phone::Symbol("c")]),
        format!("a {ESCAPE_CHAR}b c")
    );

    assert_eq!(
        phone_list_to_string(&[Phone::Symbol(&BOUND_CHAR.to_string())]),
        BOUND_CHAR.to_string()
    );
}

#[test]
fn text_matches_text () {
    assert!(Phone::Symbol("test").matches(&Phone::Symbol("test")));
    assert!(!Phone::Symbol("test").matches(&Phone::Symbol("not test")));
}

// #[test]
// fn escapes() {
//     assert!(Phone::Symbol("\\@").matches(&Phone::Symbol("@")));
//     assert!(!Phone::Symbol("@").matches(&Phone::Symbol("\\@")));
//     assert!(Phone::Symbol("\\\\@").matches(&Phone::Symbol("\\@")));
//     assert!(!Phone::Symbol("\\@").matches(&Phone::Symbol("\\@")));
//     assert!(!Phone::Symbol(&format!("\\{BOUND_STR}")).matches(&Phone::Bound));
//     assert!(!Phone::Bound.matches(&Phone::Symbol(&format!("\\{BOUND_STR}"))));
// }

#[test]
fn bound_matches_phone() {
    assert!(Phone::Symbol(" ").matches(&Phone::Bound));
    assert!(Phone::Symbol("  ").matches(&Phone::Bound));
    assert!(Phone::Bound.matches(&Phone::Symbol(" ")));
    assert!(Phone::Symbol(" ").matches(&Phone::Symbol(" ")));
}