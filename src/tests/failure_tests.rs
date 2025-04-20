use crate::{apply_fallible, apply_fallible_with_runtime, runtime::{LineApplicationLimit, Runtime}};

#[test]
fn time_out_of_infinte_loop() {
    assert!(apply_fallible("a", "{a, b} > {b, a}").is_err());
    assert!(apply_fallible_with_runtime("a", "{a, b} > {b, a}", Runtime::default().set_line_application_limit(LineApplicationLimit::Attempts(1000))).is_err());
}

#[test]
fn gap_out_of_cond() {
    assert!(apply_fallible("a", "a .. # >> b").is_err());
    assert!(apply_fallible("a", "a >> b .. c").is_err());
    assert!(apply_fallible("a", "a $gap .. # >> b $gap .. c").is_err());
}

#[test]
fn unmatched_output_scope() {
    assert!(apply_fallible("a", "a >> {b, c}").is_err());
    assert!(apply_fallible("a", "a >> (b)").is_err());
    assert!(apply_fallible("a", "a >> *").is_err());
}

#[test]
fn comma_not_in_selection() {
    assert!(apply_fallible("a", "a, b >> c").is_err());
    assert!(apply_fallible("a", "(a, b) >> c").is_err());
}

#[test]
fn invalid_labels() {
    assert!(apply_fallible("a", "$_ a >>").is_err());
    assert!(apply_fallible("a", "$_ >>").is_err());
    assert!(apply_fallible("a", "$_ $__ * >>").is_err());
}

#[test]
fn invalid_condition_tokens() {
    assert!(apply_fallible("a", "_ >>").is_err());
    assert!(apply_fallible("a", "= >>").is_err());
    assert!(apply_fallible("a", "a >> b / _ # _").is_err());
    assert!(apply_fallible("a", "a >> b / a = b = c").is_err());
}