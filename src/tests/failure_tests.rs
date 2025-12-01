use std::time::Duration;

use crate::build_rules;
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
fn repetition_out_of_cond() {
    assert!(await_io! { apply_fallible("abc", "a [*] >> b / _ #") }.is_err());
    assert!(await_io! { apply_fallible("a", "a >> b [*] c") }.is_err());
    assert!(await_io! { apply_fallible("a", "a $rep [*] # >> b $rep [*] c") }.is_err());
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

#[io_test(pollster::block_on)]
fn multi_line_errors() {
    let e = await_io! { apply_fallible("", "h >> \\\n @a") }.expect_err("should error");
    assert_eq!(e.line_num.get(), 1);
    assert_eq!(e.line_count.get(), 2);

    let e = await_io! { apply_fallible("", "h >> \\\n kh \n @a") }.expect_err("should error");
    assert_eq!(e.line_num.get(), 3);
    assert_eq!(e.line_count.get(), 1);

    let e = await_io! { apply_fallible("", "h >> \\\n <") }.expect_err("should error");
    assert_eq!(e.line_num.get(), 1);
    assert_eq!(e.line_count.get(), 2);

    let e = await_io! { apply_fallible("", "h >> \\\n kh \n >><") }.expect_err("should error");
    assert_eq!(e.line_num.get(), 3);
    assert_eq!(e.line_count.get(), 1);

    let e = await_io! { apply_fallible("h", "h >> \\\n / {a} = a") }.expect_err("should error");
    assert_eq!(e.line_num.get(), 1);
    assert_eq!(e.line_count.get(), 2);

    let e = await_io! { apply_fallible("h", "h >> \\\n kh \n kh >> / {a} = a") }.expect_err("should error");
    assert_eq!(e.line_num.get(), 3);
    assert_eq!(e.line_count.get(), 1);


    let e = await_io! { apply_fallible("", "DEFINE def h >> \\\n kh \n @a") }.expect_err("should error");
    assert_eq!(e.line_num.get(), 3);
    assert_eq!(e.line_count.get(), 1);
}


#[io_test(pollster::block_on)]
fn error_on_correct_line_after_escaped_newline_in_definition() {
    let rules = await_io! { build_rules("DEFINE def $a{\\\r\n}\r\n{a, b} >> {c}", &mut NoGet) }
        .expect("Should Build");

    assert_eq!(
        3,
        await_io! { rules.apply_fallible("b", &mut NoLog::default()) }
            .expect_err("Should Error")
            .line_number()
            .get()
    );
}