// Non rule commands executed at runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCmd {
    Print,
}

/// A log of strings
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrintLog {
    logs: Vec<String>,
}

impl PrintLog {
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