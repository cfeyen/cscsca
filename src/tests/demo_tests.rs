use crate::{apply, executor::{runtime::LogRuntime, LineByLineExecuter, getter::CliGetter}};

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
    let mut executor = LineByLineExecuter::new(LogRuntime::default(), CliGetter);
    
    _ = executor.apply_fallible("pata takan", DEMO);

    assert_eq!(
        executor.runtime().logs(),
        &[("before h-loss:".to_string(), "pat taga".to_string())]
    )
}