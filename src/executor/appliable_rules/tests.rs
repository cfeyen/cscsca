use super::*;
use crate::{io_macros::{await_io, io_test}, tests::{NoGet, NoLog}, ONE};

#[io_test(pollster::block_on)]
fn appliable_rules() {
    let rules = "DEFINE V {i, e, a, u, o}\n{p, t, k} >> {b, d, g} / @V _ @V\n@V >> / _ #";

    let appliable_rules = await_io! {
        build_rules(rules, &mut NoGet)
    }.expect("rules should compile");

    assert_eq!(
        await_io! { appliable_rules.apply_fallible("pata takan", &mut NoLog::default()) },
        Ok("pad tagan".to_string())
    );

    assert_eq!(
        await_io! { appliable_rules.apply_fallible("In a hole in the ground there lived a hobbit", &mut NoLog::default()) },
        Ok("In hol in th ground ther lived hobbit".to_string())
    );

    assert_eq!(
        await_io! { appliable_rules.apply_fallible("I ate another test", &mut NoLog::default()) },
        Ok("I ad another test".to_string())
    );
}

#[io_test(pollster::block_on)]
fn appliable_rule_runtime_errors() {
    let rules = "{a, b} >> {c}";

    let appliable_rules = await_io! { build_rules(rules, &mut NoGet) }
        .expect("rules should compile");

    let result = await_io! { appliable_rules.apply_fallible("b", &mut NoLog::default()) };
    assert!(result.is_err_and(|e| e.rule == rules && e.line_num == ONE));
}

#[io_test(pollster::block_on)]
fn appliable_rule_build_time_errors() {
    let rules = "a > b > c";

    let result = await_io! { build_rules(rules, &mut NoGet) };
    assert!(result.is_err_and(|e| e.rule == rules && e.line_num == ONE));
}

#[io_test(pollster::block_on)]
fn extend_rules() {
    let rules_1 = "a >> bc";
    let rules_2 = "bc >> d";

    let mut rules = await_io! { build_rules(rules_1, &mut NoGet) }.expect("Rules should be valid");
    let rules_extension = await_io! { build_rules(rules_2, &mut NoGet) }.expect("Rules should be valid");
    
    rules.extend(rules_extension);

    let output = await_io! { rules.apply_fallible("a bc", &mut NoLog::default()) }.expect("Rules should be valid");

    assert_eq!(&output, "d bc");
}