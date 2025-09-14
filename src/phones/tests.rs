use crate::tests::apply;
use crate::{escaped_strings::EscapedString, keywords::BOUND_CHAR};
use crate::io_macros::{await_io, io_test};

use super::*;

#[io_test(pollster::block_on)]
fn shifting_bounds() {
    assert_eq!(
        await_io! { apply("a #c", "\\# >> b") },
        "a bc"
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
        vec![Phone::Symbol("a"), Phone::Symbol(&format!("{ESCAPE_CHAR}{BOUND_CHAR}")), Phone::Symbol("b"), Phone::Bound, Phone::Symbol("c")]
    );
}

#[test]
fn phone_list_to_str() {
    assert_eq!(
        phone_list_to_string(&[Phone::Symbol("a"), Phone::Symbol("b"), Phone::Symbol("c")]),
        "abc"
    );

    assert_eq!(
        phone_list_to_string(&[Phone::Symbol("a"), Phone::Bound, Phone::Symbol("b"), Phone::Bound, Phone::Symbol("c")]),
        "a b c"
    );

    assert_eq!(
        phone_list_to_string(&[Phone::Symbol(char_to_str(&BOUND_CHAR))]),
        char_to_str(&BOUND_CHAR)
    );
}

#[test]
fn phone_list_to_str_with_escapes() {
    assert_eq!(
        phone_list_to_string(&[Phone::Symbol(&format!("{ESCAPE_CHAR}.")), Phone::Symbol("b"), Phone::Symbol("c")]),
        ".bc"
    );

    assert_eq!(
        phone_list_to_string(&[Phone::Symbol(&format!("a{ESCAPE_CHAR}.")), Phone::Bound, Phone::Symbol("b"), Phone::Bound, Phone::Symbol("c")]),
        "a. b c"
    );

    assert_eq!(
        phone_list_to_string(&[Phone::Symbol("a"), Phone::Bound, Phone::Symbol(&format!("{ESCAPE_CHAR}b")), Phone::Bound, Phone::Symbol("c")]),
        "a b c"
    );

    assert_eq!(
        phone_list_to_string(&[Phone::Symbol("a"), Phone::Bound, Phone::Symbol(&format!("{ESCAPE_CHAR}{ESCAPE_CHAR}b")), Phone::Bound, Phone::Symbol("c")]),
        format!("a {ESCAPE_CHAR}b c")
    );

    assert_eq!(
        phone_list_to_string(&[Phone::Symbol(char_to_str(&BOUND_CHAR))]),
        char_to_str(&BOUND_CHAR)
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