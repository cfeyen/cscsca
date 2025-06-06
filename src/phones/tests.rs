use crate::{apply, escaped_strings::EscapedString, keywords::BOUND_CHAR};

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
        build_phone_list(EscapedString::from("a b c").as_escaped_str()),
        vec![Phone::Symbol("a"), Phone::Bound, Phone::Symbol("b"), Phone::Bound, Phone::Symbol("c")]
    );

    assert_eq!(
        build_phone_list(EscapedString::from("ab c").as_escaped_str()),
        vec![Phone::Symbol("a"), Phone::Symbol("b"), Phone::Bound, Phone::Symbol("c")]
    );

    assert_eq!(
        build_phone_list(EscapedString::from(format!("a{BOUND_CHAR}b c").as_str()).as_escaped_str()),
        vec![Phone::Symbol("a"), Phone::Symbol(&format!("{ESCAPE_CHAR}{BOUND_CHAR}").to_string()), Phone::Symbol("b"), Phone::Bound, Phone::Symbol("c")]
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
        phone_list_to_string(&[Phone::Symbol(&BOUND_CHAR.to_string())]),
        BOUND_CHAR.to_string()
    );
}

#[test]
fn phone_list_to_str_with_escapes() {
    assert_eq!(
        phone_list_to_string(&[Phone::Symbol(&format!("{ESCAPE_CHAR}.")), Phone::Symbol("b"), Phone::Symbol("c")]),
        ".bc".to_string()
    );

    assert_eq!(
        phone_list_to_string(&[Phone::Symbol(&format!("a{ESCAPE_CHAR}.")), Phone::Bound, Phone::Symbol("b"), Phone::Bound, Phone::Symbol("c")]),
        "a. b c".to_string()
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

#[test]
fn bound_matches_phone() {
    assert!(Phone::Symbol(" ").matches(&Phone::Bound));
    assert!(Phone::Symbol("  ").matches(&Phone::Bound));
    assert!(Phone::Bound.matches(&Phone::Symbol(" ")));
    assert!(Phone::Symbol(" ").matches(&Phone::Symbol(" ")));
}