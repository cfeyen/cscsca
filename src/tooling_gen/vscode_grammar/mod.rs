use std::{fmt::Display, fs, io::Error};

use crate::keywords::{AND_CHAR, ANY_CHAR, ARG_SEP_CHAR, BOUND_CHAR, COMMENT_LINE_START, COND_CHAR, DEFINITION_LINE_START, DEFINITION_PREFIX, DOT_CHAR, ESCAPE_CHAR, GET_AS_CODE_LINE_START, GET_LINE_START, LABEL_PREFIX, LTR_CHAR, MATCH_CHAR, OPTIONAL_END_CHAR, OPTIONAL_START_CHAR, PRINT_LINE_START, RTL_CHAR, SELECTION_END_CHAR, SELECTION_START_CHAR, SQUARE_END_CHAR, SQUARE_START_CHAR, UNDERSCORE_CHAR, VARIABLE_PREFIX};

const ESCAPE: &str = "\\\\";
const LINE_START: char = '^';
const LINE_END: char = '$';
const GROUP_START: char = '(';
const GROUP_END: char = ')';
const WHITESPACE: &str = "\\\\s";
const NON_WHITESPACE: &str = "\\\\S";
const OR: char = '|';
const ANY: char = '.';
const REP_ANY: char = '*';
const REP_ONCE_PLUS: char = '+';
const REP_MAYBE_ONCE: char = '?';

#[cfg(unix)]
const LIGHT_ICON: &[u8] = include_bytes!("assets/icons/light.png");
#[cfg(windows)]
const LIGHT_ICON: &[u8] = include_bytes!("assets\\icons\\light.png");

#[cfg(unix)]
const DARK_ICON: &[u8] = include_bytes!("assets/icons/dark.png");
#[cfg(windows)]
const DARK_ICON: &[u8] = include_bytes!("assets\\icons\\dark.png");

#[cfg(unix)]
const PACKAGE_JSON: &[u8] = include_bytes!("assets/package.json");
#[cfg(windows)]
const PACKAGE_JSON: &[u8] = include_bytes!("assets\\package.json");

#[cfg(unix)]
const PATH_SEP: char = '/';
#[cfg(windows)]
const PATH_SEP: char = '\\';

pub fn gen_vscode_grammar(path: &str) -> Result<(), Error> {
    fs::create_dir(format!("{path}"))?;
    fs::create_dir(format!("{path}{PATH_SEP}icons"))?;
    fs::write(format!("{path}{PATH_SEP}icons{PATH_SEP}light.png"), LIGHT_ICON)?;
    fs::write(format!("{path}{PATH_SEP}icons{PATH_SEP}dark.png"), DARK_ICON)?;
    fs::create_dir(format!("{path}{PATH_SEP}syntaxes"))?;
    fs::write(format!("{path}{PATH_SEP}syntaxes{PATH_SEP}grammar.json"), build_grammar())?;
    fs::write(format!("{path}{PATH_SEP}config.json"), build_config())?;
    fs::write(format!("{path}{PATH_SEP}package.json"), PACKAGE_JSON)?;

    Ok(())
}

fn style_json(json: &str) -> String {
    let mut depth = 0;
    let mut styled = String::new();

    for line in json.lines().map(str::trim) {
        if line.starts_with(['}', ']']) {
            depth -= 1;
        }

        for _ in 0..depth {
            styled.push('\t');
        }

        styled += line;

        if line.ends_with(['{', '[']) {
            depth += 1;
        }

        styled.push('\n')
    }

    styled
}

fn build_config() -> String {
    let corresponding_scope_bound_list = format!(
        "[
        [\"{OPTIONAL_START_CHAR}\", \"{OPTIONAL_END_CHAR}\"],
        [\"{SELECTION_START_CHAR}\", \"{SELECTION_END_CHAR}\"]
        ]"
    );

    style_json(&format!(
        "{{
        \"comments\": {{
            \"lineComment\": \"{COMMENT_LINE_START}\"
        }},
        \"brackets\": {corresponding_scope_bound_list},
        \"autoClosingPairs\": {corresponding_scope_bound_list},
        \"surroundingPairs\": {corresponding_scope_bound_list}
        }}"
    ))
}

fn build_grammar() -> String {
    let mut patterns = Vec::new();
    let breaking = breaking_chars();
    let break_ahead = look_ahead(&breaking);

    patterns.push((
        "comment",
        PatternList {
            list: vec![
                Pattern::new("comment_", vec!["comment"], format!(
                    "{GROUP_START}{LINE_START}{}{OR}{}{GROUP_END}{ANY}{REP_ANY}{REP_MAYBE_ONCE}{LINE_END}", COMMENT_LINE_START.chars().fold(String::new(), |acc, elem| format!("{acc}{ESCAPE}{elem}")), look_behind(&format!("{LINE_START}{GROUP_START}{PRINT_LINE_START}{OR}{GROUP_START}{GET_LINE_START}{OR}{GET_AS_CODE_LINE_START}{GROUP_END}{WHITESPACE}{REP_ANY}{NON_WHITESPACE}{REP_ONCE_PLUS}{GROUP_END}")))
                ),
            ]
        }
    ));
    patterns.push((
        "expression",
        PatternList {
            list: vec![
                Pattern::new("statement", vec!["keyword", "strong"], format!("{LINE_START}{GROUP_START}{DEFINITION_LINE_START}{OR}{PRINT_LINE_START}{OR}{GET_LINE_START}{OR}{GET_AS_CODE_LINE_START}{GROUP_END}")),
                Pattern::new("escape", vec!["constant.character.escape"], format!("{ESCAPE}\\{ESCAPE_CHAR}{ANY}")),
                Pattern::new("potentially_special_characters", vec!["keyword"], format!("{}{GROUP_START}{UNDERSCORE_CHAR}{OR}{ESCAPE}{DOT_CHAR}{ESCAPE}{DOT_CHAR}{GROUP_END}{break_ahead}", look_behind(&breaking))),
                Pattern::new("special_characters", vec!["keyword"], format!("{ESCAPE}{BOUND_CHAR}{OR}{ESCAPE}{ANY_CHAR}{OR}{MATCH_CHAR}")),
                Pattern::new("definition_call", vec!["entity.name.type"], format!("{DEFINITION_PREFIX}{NON_WHITESPACE}{REP_ONCE_PLUS}{REP_MAYBE_ONCE}{break_ahead}")),
                Pattern::new("variable_call", vec!["entity.name.type", "emphasis"], format!("{VARIABLE_PREFIX}{NON_WHITESPACE}{REP_ONCE_PLUS}{REP_MAYBE_ONCE}{break_ahead}")),
                Pattern::new("label", vec!["entity.name.function", "emphasis"], format!("{ESCAPE}{LABEL_PREFIX}{NON_WHITESPACE}{REP_ONCE_PLUS}{REP_MAYBE_ONCE}{break_ahead}")),
                Pattern::new("definition_name", vec!["entity.name.type"], format!("{}{NON_WHITESPACE}{REP_ONCE_PLUS}{REP_MAYBE_ONCE}{break_ahead}", look_ahead(&format!("{LINE_START}{DEFINITION_LINE_START}{WHITESPACE}{REP_ANY}{REP_MAYBE_ONCE}")))),
                Pattern::new("variable_name", vec!["entity.name.type", "emphasis"], format!("{}{NON_WHITESPACE}{REP_ONCE_PLUS}{REP_MAYBE_ONCE}{break_ahead}", look_ahead(&format!("{LINE_START}{GROUP_START}{GET_LINE_START}{OR}{GET_AS_CODE_LINE_START}{GROUP_END}{WHITESPACE}{REP_ANY}{REP_MAYBE_ONCE}")))),
                Pattern::new("breaks", vec!["keyword.control"], format!("{LTR_CHAR}{LTR_CHAR}{OR}{LTR_CHAR}{OR}{RTL_CHAR}{RTL_CHAR}{OR}{RTL_CHAR}{OR}{COND_CHAR}{OR}{COND_CHAR}{COND_CHAR}{OR}{AND_CHAR}")),
                Pattern::new("punctuation", vec!["punctuation.seperator"], format!("{ARG_SEP_CHAR}")),
                Pattern::new("scope_bound", vec!["punctuation.bound"], format!("{ESCAPE}{OPTIONAL_START_CHAR}{OR}{ESCAPE}{OPTIONAL_END_CHAR}{OR}{ESCAPE}{SELECTION_START_CHAR}{OR}{ESCAPE}{SELECTION_END_CHAR}")),
                Pattern::new("reserved", vec!["invalid.reserved"], format!("{ESCAPE}{DOT_CHAR}{OR}{ESCAPE}{SQUARE_START_CHAR}{OR}{ESCAPE}{SQUARE_END_CHAR}")),
                Pattern::new("phone", vec!["variable.phone"], format!("{NON_WHITESPACE}{REP_ONCE_PLUS}{REP_MAYBE_ONCE}{break_ahead}")),
            ]
        }
    ));

    build_grammar_file(&patterns)
}

fn build_grammar_file(patterns: &[(&str, PatternList)]) -> String {
    let repo_pats = patterns.iter()
        .map(|(name, _)| format!("{{\"include\": \"#{name}\"}}"))
        .reduce(|acc, elem| format!("{acc}, {elem}"))
        .unwrap_or_default();

    let pat_headers = patterns.iter()
        .map(|(name, pat_list)| format!("\"{name}\": {{\n{pat_list}\n}}"))
        .reduce(|acc, elem| format!("{acc},\n{elem}"))
        .unwrap_or_default();

    let pats = patterns.iter()
        .map(|(_, pat_list)| {
            pat_list.list.iter()
                .map(Pattern::to_string)
                .reduce(|acc, elem| format!("{acc},\n{elem}"))
                .unwrap_or_default()
        })
        .reduce(|acc, elem| format!("{acc},\n{elem}"))
        .unwrap_or_default();

    style_json(&format!(
        "{{
        \"$schema\": \"https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json\",
        \"name\": \"CSCSCA\",
        \"patterns\": [{repo_pats}],
        \"repository\": {{
            {pat_headers},
            {pats}
        }},
        \"scopeName\": \"source.sca\"
        }}"
    ))
}

fn breaking_chars() -> String {
    format!("{GROUP_START}{WHITESPACE}{OR}{LINE_START}{OR}{LINE_END}{OR}{LTR_CHAR}{OR}{RTL_CHAR}{OR}{COND_CHAR}{OR}{AND_CHAR}{OR}{DEFINITION_PREFIX}{OR}{ESCAPE}{LABEL_PREFIX}{OR}{VARIABLE_PREFIX}{OR}{ESCAPE}{BOUND_CHAR}{OR}{ARG_SEP_CHAR}{OR}{ESCAPE}{ANY_CHAR}{OR}{MATCH_CHAR}{OR}{ESCAPE}{OPTIONAL_START_CHAR}{OR}{ESCAPE}{OPTIONAL_END_CHAR}{OR}{ESCAPE}{SELECTION_START_CHAR}{OR}{ESCAPE}{SELECTION_END_CHAR}{OR}{ESCAPE}{DOT_CHAR}{OR}{ESCAPE}{SQUARE_START_CHAR}{OR}{ESCAPE}{SQUARE_END_CHAR}{GROUP_END}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PatternList<'p> {
    list: Vec<Pattern<'p>>
}

impl Display for PatternList<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.list
            .iter()
            .map(|pat| format!("{{\"include\": \"#{}\" }}", pat.name))
            .reduce(|acc, elem| format!("{acc},\n{elem}"))
            .unwrap_or_default();

        write!(f, "\"patterns\": [\n{s}\n]")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pattern<'a> {
    name: &'a str,
    pat_name: Vec<&'a str>,
    match_regex: String,
}

impl<'a> Pattern<'a> {
    fn new(name: &'a str, pat_name: Vec<&'a str>, match_regex: String) -> Self {
        Self {
            name,
            pat_name,
            match_regex,
        }
    }
}

impl Display for Pattern<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\": {{\n\"name\": \"{}.cscsca\",\n\"match\": \"{}\"\n}}", self.name, self.pat_name.join(".cscsca "), self.match_regex)
    }
}

fn look_behind(s: &str) -> String {
    format!("{GROUP_START}?<={GROUP_START}{s}{GROUP_END}{GROUP_END}")
}

fn look_ahead(s: &str) -> String {
    format!("{GROUP_START}?={GROUP_START}{s}{GROUP_END}{GROUP_END}")
}