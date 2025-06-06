#[cfg(feature = "ansi")]
pub use ansi::*;

#[cfg(feature = "ansi")]
mod ansi {
    #![allow(dead_code)]
    
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[93m";
    pub const GREEN: &str = "\x1b[92m";
    pub const BLUE: &str = "\x1b[94m";
}

#[cfg(not(any(feature = "ansi")))]
pub use colorless::*;

#[cfg(not(any(feature = "ansi")))]
mod colorless {
    #![allow(dead_code)]

    pub const RESET: &str = "";
    pub const BOLD: &str = "";
    pub const RED: &str = "";
    pub const YELLOW: &str = "";
    pub const GREEN: &str = "";
    pub const BLUE: &str = "";
}