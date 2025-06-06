use crate::{apply, apply_fallible, executor::{LineByLineExecuter, runtime::CliRuntime, getter::IoGetter}};

mod demo_tests;
mod failure_tests;

#[test]
fn escape() {
    assert_eq!("Aa@", apply("@aa@", "\\@ a >> A"));
}

#[test]
fn input_escape() {
    assert_eq!("bb", apply("..", "\\. >> b"));
}

#[test]
fn reserved_chars() {
    assert!(apply_fallible("..", ". >> b").is_err());
}

struct SingleInputRuntime(&'static str);

impl IoGetter for SingleInputRuntime {
    fn get_io(&mut self, _: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.0.to_string())
    }
}

#[test]
fn input() {
    let runtime = CliRuntime::default();

    assert_eq!(
        LineByLineExecuter::new(runtime, SingleInputRuntime("a"))
            .apply_fallible("a", "GET a :\n%a >> b"),
        Ok("b".to_string())
    );

    assert_eq!(
        LineByLineExecuter::new(runtime, SingleInputRuntime("b"))
            .apply_fallible("a", "GET a :\n%a >> b"),
        Ok("a".to_string())
    );

    let mut executor = LineByLineExecuter::new(runtime, SingleInputRuntime("a >> b"));
    
    assert!(
        executor.apply_fallible("a", "GET rule :\n%rule").is_err()
    );
    
    assert_eq!(
        executor.apply_fallible("a", "GET_AS_CODE rule :\n%rule"),
        Ok("b".to_string())
    );
}

#[test]
fn matches_with_option_that_can_insert_but_should_not() {
    assert_eq!(
        "paa".to_string(),
        apply("pea", "{i, e} >> {e, $env{e, a}} / _ (*) $env{e, a}")
    );
}

#[test]
fn matches_with_selection_that_can_insert_first_but_should_insert_second() {
    assert_eq!(
        "cdeg".to_string(),
        apply("adeg", "a >> $env{b, c} / _ $env{d e, d} $env{f, e g}")
    );
}

#[test]
fn multi_phone_shift() {
    assert_eq!("efg", apply("abc", "a b c >> e f g"));
    assert_eq!("efg", apply("abc", "a b c << e f g"));

    assert_eq!("zefg", apply("zabc", "a b c >> e f g / z _"));
    assert_eq!("zefg", apply("zabc", "a b c << e f g / z _"));
    
    assert_eq!("efgz", apply("abcz", "a b c >> e f g / _ z"));
    assert_eq!("efgz", apply("abcz", "a b c << e f g / _ z"));
}

#[test]
fn escape_printing() {
    assert_eq!("\\", apply("\\", ""));
    assert_eq!("\\", apply("\\", "\\\\ >> \\\\"));
    assert_eq!("a*", apply("a", "a >> a\\*"));
    assert_eq!(apply("#", "* >> *"), apply("#", "\\# >> \\#"));
}