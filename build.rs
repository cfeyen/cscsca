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
    let template = include_str!("docs/README_template.md");
    let writing_rules = include_str!("docs/writing_rules.md");

    if !template.contains(WRITING_RULES_PLACEHOLDER) {
        panic!("Could not find '{WRITING_RULES_PLACEHOLDER}' in the template file");
    }

    let readme = template.replace(WRITING_RULES_PLACEHOLDER, writing_rules);

    fs::write("README.md", readme)
}