use crate::io_macros::{await_io, io_test};
use crate::{tests::{NoLog, NoGet}, executor::{LineByLineExecutor, runtime::LineApplicationLimit}};
use std::time::Duration;

#[io_test(pollster::block_on)]
fn time_out_of_infinte_loop() {
    let mut exector = LineByLineExecutor::new(
        NoLog::new(Some(LineApplicationLimit::Time(Duration::from_millis(100)))),
        NoGet
    );

    assert!(await_io! { exector.apply_fallible("a", "{a, b} > {b, a}") }.is_err());
}

#[io_test(pollster::block_on)]
fn does_not_time_out_of_finte_shift() {
    let mut exector = LineByLineExecutor::new(
        NoLog::new(Some(LineApplicationLimit::Time(Duration::from_millis(100)))),
        NoGet
    );

    assert!(await_io! { exector.apply_fallible("a", "{a, b} > {b, c}") }.is_ok());
}