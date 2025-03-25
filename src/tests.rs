use super::*;

#[test]
fn demo_merge_n_g_and_nasals_dropped_word_finally() {
    assert_eq!(
        String::new(),
        apply("ng", demo()).0
    );
}

#[test]
fn demo_stop_voicing_and_vowel_lostt() {
    assert_eq!(
        "p ab ed dl htl ant".to_string(),
        apply("pe apa eti tl htl ante", demo()).0
    );
}

#[test]
fn demo_stop_assimilation() {
    assert_eq!(
        "pat taga".to_string(),
        apply("pata takan", demo()).0
    );
}

#[test]
fn demo_h_loss() {
    assert_eq!(
        "h_ _".to_string(),
        apply("h_ _h", demo()).0
    );
}

#[test]
fn demo_palatalization() {
    assert_eq!(
        "taɲtʃil aɲi".to_string(),
        apply("tantil anim", demo()).0
    );
}

#[test]
fn demo_harmony() {
    assert_eq!(
        "iny iwu iwiny".to_string(),
        apply("inuh iwuh iwinuh", demo()).0
    );
}

#[test]
fn demo_print() {
    assert_eq!(
        &[format!("before h-loss: '{BLUE}pat taga{RESET}'")],
        apply("pata takan", demo()).1.logs()
    )
}

#[test]
fn escape() {
    assert_eq!(
        "Aa@",
        apply("@aa@", "\\@ a >> A").0
    )
}

#[test]
fn time_out_of_infinte_loop() {
    assert!(apply_fallible("a", "{a, b} > {b, a}").0.is_err());
}

#[test]
fn matches_with_option_that_can_insert_but_should_not() {
    assert_eq!(
        "paa".to_string(),
        apply("pea", "{i, e} >> {e, $env{e, a}} / _ (*) $env{e, a}").0
    );
}

#[test]
fn matches_with_selection_that_can_insert_first_but_should_insert_second() {
    assert_eq!(
        "cdeg".to_string(),
        apply("adeg", "a >> $env{b, c} / _ $env{d e, d} $env{f, e g}").0
    );
}