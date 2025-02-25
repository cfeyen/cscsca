use super::*;
use tokio::runtime::Runtime;
use tokio::time::{timeout, Duration};

#[test]
fn demo_merge_n_g_and_nasals_dropped_word_finally() {
    assert_eq!(
        String::new(),
        Runtime::new().unwrap().block_on(apply("ng", demo())).0
    );
}

#[test]
fn demo_stop_voicing_and_vowel_lostt() {
    assert_eq!(
        "p ab ed dl htl ant".to_string(),
        Runtime::new().unwrap().block_on(apply("pe apa eti tl htl ante", demo())).0
    );
}

#[test]
fn demo_stop_assimilation() {
    assert_eq!(
        "pat taga".to_string(),
        Runtime::new().unwrap().block_on(apply("pata takan", demo())).0
    );
}

#[test]
fn demo_h_loss() {
    assert_eq!(
        "h_ _".to_string(),
        Runtime::new().unwrap().block_on(apply("h_ _h", demo())).0
    );
}

#[test]
fn demo_palatalization() {
    assert_eq!(
        "taɲtʃil aɲi".to_string(),
        Runtime::new().unwrap().block_on(apply("tantil anim", demo())).0
    );
}

#[test]
fn demo_harmony() {
    assert_eq!(
        "iny iwu iwiny".to_string(),
        Runtime::new().unwrap().block_on(apply("inuh iwuh iwinuh", demo())).0
    );
}

#[test]
fn demo_print() {
    assert_eq!(
        &[format!("before h-loss: '{BLUE}pat taga{RESET}'")],
        Runtime::new().unwrap().block_on(apply("pata takan", demo())).1.logs()
    )
}

#[test]
fn time_out_infinite_loop() {
    assert!(
        Runtime::new().unwrap().block_on(async { timeout(
            Duration::from_millis(100),
            apply("a", "{a, b} > {b, a}")
        ).await }).is_err()
    );
}