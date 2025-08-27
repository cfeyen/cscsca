
/// Defines many constants and a list that contains all of them
/// 
/// Structure:
/// 
/// List, types and visibility:
///  
/// *`list_visiblitiy`* *`list_name`*: \[*`const visibility`* *`const type`*\];
/// 
/// Constants:
/// 
/// *`const_name`* = *`expression`*;
macro_rules! const_list {
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

const_list! {
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
    NOT_CHAR = '!';

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
    BOUND_CHAR = '#';
}

const_list! {
    UNUSED_CHARS: [pub char];

    DOT_CHAR = '.';
    SQUARE_START_CHAR = '[';
    SQUARE_END_CHAR = ']';
}

const_list! {
    ISOLATED_CHARS: [pub char];
    
    // Chars that should only be escaped when isolated
    UNDERSCORE_CHAR = '_';
}

const_list! {
    pub(crate) SPECIAL_STRS: [pub &str];

    // Strings that are only special when isolated
    GAP_STR = "..";
    INPUT_PATTERN_STR = "_";
}

/// Checks if a char can/should always be escaped
pub(crate) fn is_special_char(c: char) -> bool {
    SPECIAL_CHARS.contains(&c) || UNUSED_CHARS.contains(&c)
}

/// Checks if a char can/should be escaped when isolated
pub(crate) fn is_isolated_char(c: char) -> bool {
    ISOLATED_CHARS.contains(&c)
}

/// Checks if a char can act as abound by an isolated char
pub(crate) fn is_isolation_bound(c: char) -> bool {
    c.is_whitespace() || is_special_char(c)
}

// Strings that are only special at the start of a line
pub const DEFINITION_LINE_START: &str = "DEFINE";
pub const PRINT_LINE_START: &str = "PRINT";
pub const GET_LINE_START: &str = "GET";
pub const GET_AS_CODE_LINE_START: &str = "GET_AS_CODE";
pub const COMMENT_LINE_START: &str = "##";

/// Converts a `&char` to `&str`
pub const fn char_to_str(c: &char) -> &str {
    let ptr = std::ptr::from_ref(c).cast::<u8>();
    let utf8 = std::ptr::slice_from_raw_parts(ptr, c.len_utf8());

    // Safety: `utf8` is a valid pointer to vaild utf8
    unsafe { str::from_utf8_unchecked(&*utf8) }
}