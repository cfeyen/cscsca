//! Runs before `cscsca` is built
//! 
//! Builds the full `README.md` file

use std::{fs, io};

fn main() {
    create_readme().expect("Failed to generate README");
}

const WRITING_RULES_PLACEHOLDER: &str = "[Writing Rules]";

/// Builds the final `README.md` file from components in the docs folder
fn create_readme() -> io::Result<()> {
    let frame = include_str!("docs/README_frame.md");
    let writing_rules = include_str!("docs/writing_rules.md");

    let readme = frame.replace(WRITING_RULES_PLACEHOLDER, writing_rules);

    if &readme == frame {
        panic!("Could not find '{WRITING_RULES_PLACEHOLDER}' in the template file");
    }

    fs::write("README.md", readme)
}