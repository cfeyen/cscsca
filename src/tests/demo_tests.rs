use crate::{executor::{runtime::LogRuntime, LineByLineExecuter}, tests::{apply, NoGet}};
use crate::io_macros::{await_io, io_test};

const DEMO: &'static str = include_str!("../assets/demo.sca");

#[io_test(pollster::block_on)]
fn demo_merge_n_g_and_nasals_dropped_word_finally() {
    assert_eq!(
        String::new(),
        await_io! { apply("ng", DEMO) }
    );
}

#[io_test(pollster::block_on)]
fn demo_stop_voicing_and_vowel_lost() {
    assert_eq!(
        "p ab ed dl htl ant",
        await_io! { apply("pe apa eti tl htl ante", DEMO) }
    );
}

#[io_test(pollster::block_on)]
fn demo_stop_assimilation() {
    assert_eq!(
        "pat taga",
        await_io! { apply("pata takan", DEMO) }
    );
}

#[io_test(pollster::block_on)]
fn demo_h_loss() {
    assert_eq!(
        "h_ _",
        await_io! { apply("h_ _h", DEMO) }
    );
}

#[io_test(pollster::block_on)]
fn demo_palatalization() {
    assert_eq!(
        "taɲtʃil aɲi",
        await_io! { apply("tantil anim", DEMO) }
    );
}

#[io_test(pollster::block_on)]
fn demo_harmony() {
    assert_eq!(
        "iny iwu iwiny",
        await_io! { apply("inuh iwuh iwinuh", DEMO) }
    );
}

#[io_test(pollster::block_on)]
fn demo_print() {
    let mut executor = LineByLineExecuter::new(LogRuntime::default(), NoGet);
    
    _ = await_io! { executor.apply_fallible("pata takan", DEMO) };

    assert_eq!(
        executor.runtime().logs(),
        &[("before h-loss:".to_string(), "pat taga".to_string())]
    )
}
