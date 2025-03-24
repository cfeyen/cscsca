use std::collections::HashMap;

use crate::{*, applier::async_applier};
use tokio::time::{timeout, Duration, error::Elapsed};

#[cfg(test)]
mod async_test;

/// Runs `async_cscsca::apply` for a finite time
pub async fn limited_apply(input: &str, code: &str, limit: Duration) -> Result<(String, PrintLog), Elapsed> {
    timeout(limit, apply(input, code)).await
}

/// Runs `async_cscsca::apply_fallible` for a finite time
pub async fn limited_apply_fallible(input: &str, code: &str, print_log: &mut PrintLog, limit: Duration) -> Result<Result<String, String>, Elapsed> {
    timeout(limit, apply_fallible(input, code, print_log)).await
}

/// Asynchronously applies sca source code to an input string
/// 
/// Returns a string of either the final text or a formatted error and the print log
// ! Should be made to remain in line with `cscsca::apply`
pub async fn apply(input: &str, code: &str) -> (String, PrintLog) {
    let mut log = PrintLog::default();
    let result = apply_fallible(input, code, &mut log).await;

    (result.unwrap_or_else(|e| e), log)
}

/// Asynchronously applies sca source code to an input string
/// 
/// Returns a result of either the final text or a formatted error and the print log
// ! Should be made to remain in line with `cscsca::apply_fallible`
pub async fn apply_fallible(input: &str, code: &str, print_log: &mut PrintLog) -> Result<String, String> {
    let mut definitions = HashMap::new();
    let lines_with_nums = code_by_line(code);
    let mut phone_list = build_phone_list(input);

    for (line_num, line) in lines_with_nums {
        let rule_line = match build_rule(line, line_num, &mut definitions) {
            Ok(line) => line,
            Err(e) => return Err(e),
        };

        match rule_line {
            RuleLine::Rule(rule) => {
                async_applier::apply(&rule, &mut phone_list)
                    .await
                    .map_err(|e| format_error(e, line, line_num))?
            },
            RuleLine::Empty => (),
            RuleLine::Cmd(cmd, args) => handle_runtime_cmd(cmd, args, &phone_list, print_log),
        }
    }

    Ok(phone_list_to_string(&phone_list))
}