use std::time::Duration;

use crate::{apply_fallible, executor::{LineByLineExecuter, runtime::{CliRuntime, LineApplicationLimit}, getter::CliGetter}};
use crate::io_macros::{await_io, io_test};

io_test! {
    fn time_out_of_infinte_loop() {
        let mut exector = LineByLineExecuter::new(
            CliRuntime::new(Some(LineApplicationLimit::Time(Duration::from_millis(100)))),
            CliGetter::new()
        );

        assert!(await_io! { exector.apply_fallible("a", "{a, b} > {b, a}") }.is_err());
    }
}

io_test! {
    fn gap_out_of_cond() {
        assert!(await_io! { apply_fallible("abc", "a .. >> b / _ #") }.is_err());
        assert!(await_io! { apply_fallible("a", "a >> b .. c") }.is_err());
        assert!(await_io! { apply_fallible("a", "a $gap .. # >> b $gap .. c") }.is_err());
    }
}

io_test! {
    fn unmatched_output_scope() {
        assert!(await_io! { apply_fallible("a", "a >> {b, c}") }.is_err());
        assert!(await_io! { apply_fallible("a", "a >> (b)") }.is_err());
        assert!(await_io! { apply_fallible("a", "a >> *") }.is_err());
    }
}

io_test! {
    fn comma_not_in_selection() {
        assert!(await_io! { apply_fallible("a", "a, b >> c") }.is_err());
        assert!(await_io! { apply_fallible("a", "(a, b) >> c") }.is_err());
    }
}

io_test! {
    fn invalid_labels() {
        assert!(await_io! { apply_fallible("a", "$_ a >>") }.is_err());
        assert!(await_io! { apply_fallible("a", "$_ >>") }.is_err());
        assert!(await_io! { apply_fallible("a", "$_ $__ * >>") }.is_err());
    }
}

io_test! {
    fn invalid_condition_tokens() {
        assert!(await_io! { apply_fallible("a", "_ >>") }.is_err());
        assert!(await_io! { apply_fallible("a", "= >>") }.is_err());
        assert!(await_io! { apply_fallible("a", "a >> b / _ # _") }.is_err());
        assert!(await_io! { apply_fallible("a", "a >> b / a = b = c") }.is_err());
    }
}