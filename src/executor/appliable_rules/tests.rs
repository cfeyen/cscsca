use super::*;
use crate::executor::{runtime::CliRuntime, getter::CliGetter};

#[test]
fn appliable_rules() {
    let rules = "DEFINE V {i, e, a, u, o}
{p, t, k} >> {b, d, g} / @V _ @V
@V >> / _ #";

    let appliable_rules = compile_rules(rules, &mut CliGetter)
        .expect("rules should compile");

    assert_eq!(
        appliable_rules.apply_fallible("pata takan", &mut CliRuntime::default()),
        Ok("pad tagan".to_string())
    );

    assert_eq!(
        appliable_rules.apply_fallible("In a hole in the ground there lived a hobbit", &mut CliRuntime::default()),
        Ok("In hol in th ground ther lived hobbit".to_string())
    );

    assert_eq!(
        appliable_rules.apply_fallible("I ate another test", &mut CliRuntime::default()),
        Ok("I ad another test".to_string())
    );
}

#[test]
fn appliable_rule_runtime_errors() {
    let rules = "{a, b} >> {c}";

    let appliable_rules = compile_rules(rules, &mut CliGetter)
        .expect("rules should compile");

    let result = appliable_rules.apply_fallible("b", &mut CliRuntime::default());
    assert!(result.is_err_and(|e| e.line == rules && e.line_num == 1));
}

#[test]
fn appliable_rule_compile_time_errors() {
    let rules = "a > b > c";

    let result = compile_rules(rules, &mut CliGetter);
    println!("{result:?}");
    assert!(result.is_err_and(|e| e.line == rules && e.line_num == 1));
}