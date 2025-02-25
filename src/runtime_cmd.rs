// Non rule commands executed at runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCmd {
    Print,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrintLogs {
    logs: Vec<String>,
}

impl PrintLogs {
    pub fn print(&self) {
        for log in &self.logs {
            println!("{log}");
        }
    }

    pub fn log(&mut self, log: String) {
        self.logs.push(log);
    }

    pub fn logs(&self) -> &[String] {
        &self.logs
    }

    pub fn flush(self) -> Vec<String> {
        self.logs
    }
}