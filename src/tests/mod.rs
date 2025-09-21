use crate::{executor::{getter::IoGetter, runtime::DEFAULT_LINE_APPLICATION_LIMIT, LineByLineExecuter}, LineApplicationLimit, Runtime, ScaError};
use crate::io_macros::{await_io, io_test, io_fn};

/// Applies rules to an input with default execution and errors converted to a string
#[io_fn]
pub fn apply(input: &str, rules: &str) -> String {
    await_io! {
        LineByLineExecuter::new(NoLog::default(), NoGet)
            .apply(input, rules)
    }
}

/// Applies rules to an input with default execution
#[io_fn]
pub fn apply_fallible(input: &str, rules: &str) -> Result<String, ScaError> {
    await_io! {
        LineByLineExecuter::new(NoLog::default(), NoGet)
            .apply_fallible(input, rules)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoGet;

impl IoGetter for NoGet {
    #[io_fn(impl)]
    fn get_io(&mut self, _: &str) -> Result<String, Box<dyn std::error::Error>> {
        return Err(Box::new(Self) as Box<dyn std::error::Error>);
    }
}

impl std::error::Error for NoGet {}

impl std::fmt::Display for NoGet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`GET` and `GET_AS_CODE` not implemented")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoLog(Option<LineApplicationLimit>);

impl NoLog {
    pub const fn new(line_application_limit: Option<LineApplicationLimit>) -> Self {
        Self(line_application_limit)
    }
}

impl Runtime for NoLog {
    fn line_application_limit(&self) -> Option<LineApplicationLimit> {
        self.0
    }

    #[io_fn(impl)]
    fn put_io(&mut self, _: &str, _:String) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

impl Default for NoLog {
    fn default() -> Self {
        Self(Some(DEFAULT_LINE_APPLICATION_LIMIT))
    }
}

mod demo_tests;
mod failure_tests;

#[io_test(pollster::block_on)]
fn escape() {
    assert_eq!("Aa@", await_io! { apply("@aa@", "\\@ a >> A") });
}

#[io_test(pollster::block_on)]
fn input_escape() {
    assert_eq!("bb", await_io! { apply("..", "\\. >> b") });
}

#[io_test(pollster::block_on)]
fn reserved_chars() {
    assert!(await_io! { apply_fallible("..", ". >> b") }.is_err());
}

#[io_test(pollster::block_on)]
fn multi_line() {
    assert_eq!("aha", await_io! { apply("hahah", "h >> \\\n / # _ \\\n / _ #") });
    assert_eq!("aha", await_io! { apply("hahah", "DEFINE cond / # _ \\\n / _ # \n h >> \\\n @cond") });
}

struct SingleInputGetter(&'static str);

impl IoGetter for SingleInputGetter {
    #[io_fn(impl)]
    fn get_io(&mut self, _: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.0.to_string())
    }
}

#[io_test(pollster::block_on)]
fn input() {
    let runtime = NoLog::default();

    assert_eq!(
        await_io! {
            LineByLineExecuter::new(runtime, SingleInputGetter("a"))
                .apply_fallible("a", "GET a :\n%a >> b")
        },
        Ok("b".to_string())
    );

    assert_eq!(
        await_io! {
            LineByLineExecuter::new(runtime, SingleInputGetter("b"))
                .apply_fallible("a", "GET a :\n%a >> b")
        },
        Ok("a".to_string())
    );

    let mut executor = LineByLineExecuter::new(runtime, SingleInputGetter("a >> b"));
    
    assert!(
        await_io! { executor.apply_fallible("a", "GET rule :\n%rule") }.is_err()
    );
    
    assert_eq!(
        await_io! { executor.apply_fallible("a", "GET_AS_CODE rule :\n%rule") },
        Ok("b".to_string())
    );
}

#[io_test(pollster::block_on)]
fn matches_with_option_that_can_insert_but_should_not() {
    assert_eq!(
        "paa",
        await_io! { apply("pea", "{i, e} >> {e, $env{e, a}} / _ (*) $env{e, a}") }
    );
}

#[io_test(pollster::block_on)]
fn matches_with_selection_that_can_insert_first_but_should_insert_second() {
    assert_eq!(
        "cdeg",
        await_io! { apply("adeg", "a >> $env{b, c} / _ $env{d e, d} $env{f, e g}") }
    );
}

#[io_test(pollster::block_on)]
fn multi_phone_shift() {
    assert_eq!("efg", await_io! { apply("abc", "a b c >> e f g") });
    assert_eq!("efg", await_io! { apply("abc", "a b c << e f g") });

    assert_eq!("zefg", await_io! { apply("zabc", "a b c >> e f g / z _") });
    assert_eq!("zefg", await_io! { apply("zabc", "a b c << e f g / z _") });
    
    assert_eq!("efgz", await_io! { apply("abcz", "a b c >> e f g / _ z") });
    assert_eq!("efgz", await_io! { apply("abcz", "a b c << e f g / _ z") });
}

#[io_test(pollster::block_on)]
fn escape_printing() {
    assert_eq!("\\", await_io! { apply("\\", "") });
    assert_eq!("\\", await_io! { apply("\\", "\\\\ >> \\\\") });
    assert_eq!("a*", await_io! { apply("a", "a >> a\\*") });
    assert_eq!(await_io! { apply("#", "* >> *") }, await_io! { apply("#", "\\# >> \\#") });
}

#[io_test(pollster::block_on)]
fn complex_agreement() {
    assert_eq!("zbc", await_io! { apply("abc", "a >> z / _ $c{b, * c} // _ $c{b, d}") });
}