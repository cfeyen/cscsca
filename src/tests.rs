use super::*;
use ansi::*;

const DEMO: &'static str = include_str!("assets/demo.sca");

#[test]
fn demo_merge_n_g_and_nasals_dropped_word_finally() {
    assert_eq!(
        String::new(),
        apply("ng", DEMO)
    );
}

#[test]
fn demo_stop_voicing_and_vowel_lost() {
    assert_eq!(
        "p ab ed dl htl ant".to_string(),
        apply("pe apa eti tl htl ante", DEMO)
    );
}

#[test]
fn demo_stop_assimilation() {
    assert_eq!(
        "pat taga".to_string(),
        apply("pata takan", DEMO)
    );
}

#[test]
fn demo_h_loss() {
    assert_eq!(
        "h_ _".to_string(),
        apply("h_ _h", DEMO)
    );
}

#[test]
fn demo_palatalization() {
    assert_eq!(
        "taɲtʃil aɲi".to_string(),
        apply("tantil anim", DEMO)
    );
}

#[test]
fn demo_harmony() {
    assert_eq!(
        "iny iwu iwiny".to_string(),
        apply("inuh iwuh iwinuh", DEMO)
    );
}

#[test]
fn demo_print() {
    use std::{rc::Rc, cell::RefCell};

    let logs = Rc::new(RefCell::new(Vec::new()));
    let logs_clone = logs.clone();

    let mut runtime = Runtime::new();
    runtime.set_io_put_fn(Box::new(move |msg| {
        logs_clone.borrow_mut().push(msg.to_string());
        Ok(())
    }));

    _ = apply_with_runtime("pata takan", DEMO, &runtime);

    assert_eq!(
        &vec![format!("before h-loss: '{BLUE}pat taga{RESET}'")],
        &*logs.borrow()
    )
}

#[test]
fn escape() {
    assert_eq!(
        "Aa@",
        apply("@aa@", "\\@ a >> A")
    )
}

#[test]
fn time_out_of_infinte_loop() {
    assert!(apply_fallible("a", "{a, b} > {b, a}").is_err());
}

#[test]
fn input() {
    assert_eq!(
        Runtime::new()
            .set_io_get_fn(Box::new(|_| Ok(String::from("a"))))
            .apply("a", "GET a :\n%a >> b")
            , Ok("b".to_string())
    );

    assert_eq!(
        Runtime::new()
            .set_io_get_fn(Box::new(|_| Ok(String::from("b"))))
            .apply("a", "GET a :\n%a >> b")
            , Ok("a".to_string())
    );
    
    assert!(
        Runtime::new()
            .set_io_get_fn(Box::new(|_| Ok(String::from("a >> b"))))
            .apply("a", "GET rule :\n%rule")
            .is_err()
    );
    
    assert_eq!(
        Runtime::new()
            .set_io_get_fn(Box::new(|_| Ok(String::from("a >> b"))))
            .apply("a", "GET_AS_CODE rule :\n%rule")
            , Ok("b".to_string())
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