use std::collections::HashMap;

use crate::{*, applier::async_applier};
use tokio::time::{timeout, Duration, error::Elapsed};

// todo: consider passing &mut logs to apply_falliable to a function that can early terminate and still return logs can be written

#[cfg(test)]
mod async_test;

/// Runs `async_cscsca::apply` for a finite time
pub async fn limited_apply(input: &str, code: &str, limit: Duration) -> Result<(String, PrintLogs), Elapsed> {
    timeout(limit, apply(input, code)).await
}

/// Runs `async_cscsca::apply_falliable` for a finite time
pub async fn limited_apply_falliable(input: &str, code: &str, limit: Duration) -> Result<(Result<String, String>, PrintLogs), Elapsed> {
    timeout(limit, apply_falliable(input, code)).await
}

/// Asynchronously applies sca source code to an input string
/// 
/// Returns a string of either the final text or a formatted error and the print logs
// ! Should be made to remain in line with `cscsca::apply`
pub async fn apply(input: &str, code: &str) -> (String, PrintLogs) {
    let (res, logs) = apply_falliable(input, code).await;
    let output = match res {
        Ok(final_phones) => final_phones,
        Err(e) => e,
    };

    (output, logs)
}

/// Asynchronously applies sca source code to an input string
/// 
/// Returns a result of either the final text or a formatted error and the print logs
// ! Should be made to remain in line with `cscsca::apply_falliable`
pub async fn apply_falliable(input: &str, code: &str) -> (Result<String, String>, PrintLogs) {
    let mut definitions = HashMap::new();
    let lines_with_nums = code_by_line(code);
    let mut phone_list = build_phone_list(input);

    let mut print_logs = PrintLogs::default();

    for (line_num, line) in lines_with_nums {
        let rule_line = match build_rule(line, line_num, &mut definitions) {
            Ok(line) => line,
            Err(e) => return (Err(e), print_logs),
        };

        match rule_line {
            RuleLine::Rule(rule) => {
                if let Err(e) = async_applier::apply(&rule, &mut phone_list)
                    .await
                    .map_err(|e| format_error(e, line, line_num)) {
                        return (Err(e), print_logs)
                    }
            }
            RuleLine::Empty => (),
            RuleLine::Cmd(cmd, args) => handle_runtime_cmd(cmd, args, &phone_list, &mut print_logs),
        }
    }

    (Ok(phone_list_to_string(&phone_list)), print_logs)
}