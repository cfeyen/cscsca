//! Runs before `cscsca` is built
//! 
//! Builds the full `README.md` file

use std::{fs, io};

fn main() {
    create_readme().expect("Failed to generate README");
}

const WRITING_RULES_PLACEHOLDER: &str = "[Writing Rules]";
const TEMPLATE: &str = include_str!("docs/README_template.md");
const WRITING_RULES: &str = include_str!("docs/writing_rules.md");

/// Builds the final `README.md` file from components in the docs folder
fn create_readme() -> io::Result<()> {
    // gets the number of times the writing rules placeholder appears in the template file
    let writing_rules_placeholder_count = TEMPLATE.split(WRITING_RULES_PLACEHOLDER).count() - 1;

    // ensures that there is exactally one occurance of the writing rules placeholder
    assert!(
        writing_rules_placeholder_count == 1,
        "found {writing_rules_placeholder_count} occurances of '{WRITING_RULES_PLACEHOLDER}' in the template file"
    );

    // replaces the writing rules placeholder with its replacement replacement
    let readme = TEMPLATE.replace(WRITING_RULES_PLACEHOLDER, WRITING_RULES);

    // writes the final output as the project's README
    fs::write("README.md", readme)
}