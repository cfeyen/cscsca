use crate::{apply, apply_with_runtime, Runtime, ansi::*};

const DEMO: &'static str = include_str!("../assets/demo.sca");

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