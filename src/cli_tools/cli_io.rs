use std::io::{self, Write as _};

use crate::cli_tools::ansi::MAGENTA;

use super::ansi::{BLUE, RESET};

use cscsca::{
    IoGetter,
    LineApplicationLimit,
    LogRuntime,
    Runtime,
};

/// A basic `IoGetter` that get input from standard input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CliGetter;

impl IoGetter for CliGetter {
    fn get_io(&mut self, msg: &str) -> Result<String, Box<dyn std::error::Error>> {
        print!("{msg} {MAGENTA}");
        let mut buffer = String::new();
        _ = io::stdout().flush();
        io::stdin().read_line(&mut buffer)?;
        print!("{RESET}");
        Ok(buffer.trim().to_string())
    }
}

/// A basic `Runtime` that logs outputs to itself and prints its logs to standard output
/// 
/// Clears its logs before starting to apply a new set of rules
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LogAndPrintRuntime(LogRuntime);

impl LogAndPrintRuntime {
    /// Returns the logs and replaces them with empty logs
    pub fn flush_logs(&mut self) -> Vec<(String, String)> {
        self.0.flush_logs()
    }
}

impl Runtime for LogAndPrintRuntime {
    fn line_application_limit(&self) -> Option<LineApplicationLimit> {
        self.0.line_application_limit()
    }
    
    fn put_io(&mut self, msg: &str, phones: String) -> Result<(), Box<dyn std::error::Error>> {
        println!("{msg} '{BLUE}{phones}{RESET}'");
        self.0.put_io(msg, phones)
    }

    fn on_start(&mut self) {
        self.0.on_start();
    }
}