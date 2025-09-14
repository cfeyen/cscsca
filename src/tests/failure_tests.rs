use std::time::Duration;

use crate::executor::{LineByLineExecuter, runtime::LineApplicationLimit};
use crate::io_macros::{await_io, io_test};
use crate::tests::{apply_fallible, NoGet, NoLog};

#[io_test(pollster::block_on)]
fn time_out_of_infinte_loop() {
    let mut exector = LineByLineExecuter::new(
        NoLog::new(Some(LineApplicationLimit::Time(Duration::from_millis(100)))),
        NoGet
    );

    assert!(await_io! { exector.apply_fallible("a", "{a, b} > {b, a}") }.is_err());
}

#[io_test(pollster::block_on)]
fn gap_out_of_cond() {
    assert!(await_io! { apply_fallible("abc", "a .. >> b / _ #") }.is_err());
    assert!(await_io! { apply_fallible("a", "a >> b .. c") }.is_err());
    assert!(await_io! { apply_fallible("a", "a $gap .. # >> b $gap .. c") }.is_err());
}

#[io_test(pollster::block_on)]
fn unmatched_output_scope() {
    assert!(await_io! { apply_fallible("a", "a >> {b, c}") }.is_err());
    assert!(await_io! { apply_fallible("a", "a >> (b)") }.is_err());
    assert!(await_io! { apply_fallible("a", "a >> *") }.is_err());
}

#[io_test(pollster::block_on)]
fn comma_not_in_selection() {
    assert!(await_io! { apply_fallible("a", "a, b >> c") }.is_err());
    assert!(await_io! { apply_fallible("a", "(a, b) >> c") }.is_err());
}

#[io_test(pollster::block_on)]
fn invalid_labels() {
    assert!(await_io! { apply_fallible("a", "$_ a >>") }.is_err());
    assert!(await_io! { apply_fallible("a", "$_ >>") }.is_err());
    assert!(await_io! { apply_fallible("a", "$_ $__ * >>") }.is_err());
}

#[io_test(pollster::block_on)]
fn invalid_condition_tokens() {
    assert!(await_io! { apply_fallible("a", "_ >>") }.is_err());
    assert!(await_io! { apply_fallible("a", "= >>") }.is_err());
    assert!(await_io! { apply_fallible("a", "a >> b / _ # _") }.is_err());
    assert!(await_io! { apply_fallible("a", "a >> b / a = b = c") }.is_err());
}