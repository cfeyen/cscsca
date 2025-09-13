use crate::{apply, apply_fallible, executor::{LineByLineExecuter, runtime::CliRuntime, getter::IoGetter}};
use crate::io_macros::{await_io, io_test, io_fn};

mod demo_tests;
mod failure_tests;

io_test! {
    fn escape() {
        assert_eq!("Aa@", await_io! { apply("@aa@", "\\@ a >> A") });
    }
}

io_test! {
    fn input_escape() {
        assert_eq!("bb", await_io! { apply("..", "\\. >> b") });
    }
}

io_test! {
    fn reserved_chars() {
        assert!(await_io! { apply_fallible("..", ". >> b") }.is_err());
    }
}

struct SingleInputGetter(&'static str);

impl IoGetter for SingleInputGetter {
    #[io_fn]
    fn get_io(&mut self, _: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.0.to_string())
    }
}

io_test! {
    fn input() {
        let runtime = CliRuntime::default();

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
}

io_test! {
    fn matches_with_option_that_can_insert_but_should_not() {
        assert_eq!(
            "paa",
            await_io! { apply("pea", "{i, e} >> {e, $env{e, a}} / _ (*) $env{e, a}") }
        );
    }
}

io_test! {
    fn matches_with_selection_that_can_insert_first_but_should_insert_second() {
        assert_eq!(
            "cdeg",
            await_io! { apply("adeg", "a >> $env{b, c} / _ $env{d e, d} $env{f, e g}") }
        );
    }
}

io_test! {
    fn multi_phone_shift() {
        assert_eq!("efg", await_io! { apply("abc", "a b c >> e f g") });
        assert_eq!("efg", await_io! { apply("abc", "a b c << e f g") });

        assert_eq!("zefg", await_io! { apply("zabc", "a b c >> e f g / z _") });
        assert_eq!("zefg", await_io! { apply("zabc", "a b c << e f g / z _") });
        
        assert_eq!("efgz", await_io! { apply("abcz", "a b c >> e f g / _ z") });
        assert_eq!("efgz", await_io! { apply("abcz", "a b c << e f g / _ z") });
    }
}

io_test! {
    fn escape_printing() {
        assert_eq!("\\", await_io! { apply("\\", "") });
        assert_eq!("\\", await_io! { apply("\\", "\\\\ >> \\\\") });
        assert_eq!("a*", await_io! { apply("a", "a >> a\\*") });
        assert_eq!(await_io! { apply("#", "* >> *") }, await_io! { apply("#", "\\# >> \\#") });
    }
}

io_test!{
    fn complex_agreement() {
        assert_eq!("zbc", await_io! { apply("abc", "a >> z / _ $c{b, * c} // _ $c{b, d}") });
    }
}