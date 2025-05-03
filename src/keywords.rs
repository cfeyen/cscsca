/// Defines many constants and a list that contains all of them
/// 
/// Structured:
/// 
/// constants! {
/// 
/// *list_visablitiy* *list_name*: \[*const_visibility* *const_type*\];
/// 
/// *const_name* = *expression*;
/// 
/// *const_name* = *expression*;
/// 
/// ...
/// 
/// }
macro_rules! constants {
    ($list_vis:vis $list:ident: [$vis:vis $t:ty]; $($name:ident = $char:expr;)*) => {
        $($vis const $name: $t = $char;)*

        $list_vis const $list: [$t; len!($($name),*)] = [$($name),*];
    };
}

/// Gets the length of a comma seperated list of identifies
macro_rules! len {
    () => {
        0
    };
    ($_:ident $(, $rest:ident)*) => {
        1 + len!($($rest),*)
    };
}

constants! {
    SPECIAL_CHARS: [pub char];
    
    // Prefixes
    DEFINITION_PREFIX = '@';
    LABEL_PREFIX = '$';
    VARIABLE_PREFIX = '%';

    // Break charactes
    LTR_CHAR = '>';
    RTL_CHAR = '<';
    COND_CHAR = '/';
    AND_CHAR = '&';

    // Scope Bounds
    OPTIONAL_START_CHAR = '(';
    OPTIONAL_END_CHAR = ')';
    SELECTION_START_CHAR = '{';
    SELECTION_END_CHAR = '}';

    // Cond foci
    MATCH_CHAR = '=';

    // other
    ANY_CHAR = '*';
    ARG_SEP_CHAR = ',';
    ESCAPE_CHAR = '\\';
}

constants! {
    SPECIAL_STRS: [pub &str];

    // Strings that are only special when isolated
    GAP_STR = "..";
    INPUT_PATTERN_STR = "_";
    BOUND_STR = "#";
}

constants! {
    _LINE_STARTS: [pub &str];

    // Strings that are only special at the start of a line
    DEFINITION_LINE_START = "DEFINE";
    PRINT_LINE_START = "PRINT";
    GET_LINE_START = "GET";
    GET_AS_CODE_LINE_START = "GET_AS_CODE";
    COMMENT_LINE_START = "##";
}


/// Determines if a character has a function to escape
pub(crate) fn is_special(c: char) -> bool {
    SPECIAL_CHARS.contains(&c)
    || SPECIAL_STRS.iter().any(|s| s.starts_with(c))
}