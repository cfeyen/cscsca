use super::{
    runtime::LogRuntime,
    getter::IoGetter,
    LineByLineExecuter,
};

use crate::{io_macros::{await_io, io_fn, io_test}, tests::{NoGet, NoLog}};

struct SingleGetter(&'static str);

impl IoGetter for SingleGetter {
    #[io_fn(impl)]
    fn get_io(&mut self, _: &str) -> Result<String, String> {
        Ok(self.0.to_string())
    }
}

#[io_test(pollster::block_on)]
fn line_by_line_getter() {
    let get_b = SingleGetter("b");

    let rules = "GET var :\na >> %var";

    assert_eq!(
        await_io! {
            LineByLineExecuter::new(NoLog::default(), get_b)
                .apply_fallible("a", rules)
        },
        Ok("b".to_string())
    );
}

#[io_test(pollster::block_on)]
fn line_by_line_log_runtime() {
    let rules = "PRINT 1:\na >> b\nPRINT 2:\nc >> d\nPRINT 3:";

    let mut executor = LineByLineExecuter::new(LogRuntime::default(), NoGet);

    assert_eq!(
        await_io! { executor.apply_fallible("abcd", rules) },
        Ok("bbdd".to_string())
    );

    assert_eq!(
        executor.runtime().logs(),
        &[
            ("1:".to_string(), "abcd".to_string()),
            ("2:".to_string(), "bbcd".to_string()),
            ("3:".to_string(), "bbdd".to_string()),
        ]
    );
}