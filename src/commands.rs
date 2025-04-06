/// Non rule commands when running a program
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Print,
    Get,
    // todo: GetAsPhone
}

/// A log of strings
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrintLog {
    logs: Vec<String>,
}

impl PrintLog {
    /// Creates an empty log
    #[inline]
    pub const fn new() -> Self {
        Self { logs: Vec::new() }
    }

    /// Prints the logs
    pub fn print(&self) {
        for log in &self.logs {
            println!("{log}");
        }
    }

    /// Logs a string
    #[inline]
    pub fn log(&mut self, log: String) {
        self.logs.push(log);
    }

    /// Returns a reference to the logs
    #[inline]
    pub fn logs(&self) -> &[String] {
        &self.logs
    }

    /// Emptys and returns the log contents
    #[inline]
    pub fn flush(self) -> Vec<String> {
        self.logs
    }
}