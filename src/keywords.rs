
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
    (
        $(#[doc = $docs:tt])*
        $list_vis:vis $list:ident: [$vis:vis $t:ty];
        $(
            $(#[doc = $const_docs:tt])*
            $name:ident = $char:expr;
        )*
    ) => {
        $(
            $(#[doc = $const_docs])*
            $vis const $name: $t = $char;
        )*

        $(#[doc = $docs])*
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
    /// A list of all charactars that are always special
    SPECIAL_CHARS: [pub char];
    
    // Prefixes
    /// The prefix for accessing a definition
    DEFINITION_PREFIX = '@';
    /// The prefix for defining a label
    LABEL_PREFIX = '$';
    /// The prefix for accessing a variable
    VARIABLE_PREFIX = '%';

    // Break charactes
    /// The character for a left-to-right shift, may be duplicated
    LTR_CHAR = '>';
    /// The character for a right-to-left shift, may be duplicated
    RTL_CHAR = '<';
    /// The character for a condition, may be duplicated for an anti-condition
    COND_CHAR = '/';
    /// The character for an and-clause in a condition
    AND_CHAR = '&';
    /// The character for negating conditions and and-clauses
    NOT_CHAR = '!';

    // Scope Bounds
    /// the start of an optional scope
    OPTIONAL_START_CHAR = '(';
    /// the end of an optional scope
    OPTIONAL_END_CHAR = ')';
    /// the start of an selection scope
    SELECTION_START_CHAR = '{';
    /// the end of an selection scope
    SELECTION_END_CHAR = '}';

    // Cond foci
    /// The seperator in a match condition
    MATCH_CHAR = '=';

    // Other
    /// Any non-bound phone
    ANY_CHAR = '*';
    /// The seperator between selection options
    ARG_SEP_CHAR = ',';
    /// Escapes special characters
    ESCAPE_CHAR = '\\';
    /// A word boundary
    BOUND_CHAR = '#';
}

const_list! {
    /// Special characters that are not used by themselves
    UNUSED_CHARS: [pub char];

    /// Used when duplicated for a gap
    DOT_CHAR = '.';
    SQUARE_START_CHAR = '[';
    SQUARE_END_CHAR = ']';
}

const_list! {
    /// Chars that should only be escaped when isolated
    ISOLATED_CHARS: [pub char];
    
    /// The input in a pattern condition
    UNDERSCORE_CHAR = '_';
}

const_list! {
    /// Strings that act like special characters when isolated
    pub(crate) SPECIAL_STRS: [pub &str];

    /// A gap
    GAP_STR = "..";
    /// The input in a pattern condition
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

/// Checks if a char can act as a bound by an isolated char
pub(crate) fn is_isolation_bound(c: char) -> bool {
    c.is_whitespace() || is_special_char(c)
}

// Strings that are only special at the start of a line
pub const DEFINITION_LINE_START: &str = "DEFINE";
pub const LAZY_DEFINITION_LINE_START: &str = "DEFINE_LAZY";
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