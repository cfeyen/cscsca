use crate::phones::Phone;

use super::*;

/// Converts source code into intermediate representation tokens
/// 
/// Note: these tokens may not be structurally valid and should be checked
/// 
/// If there is an error it is returned with the number of the line it occured on
/// 
/// ## Warning
/// Any io fetched during application will be leaked to the static scope
#[cfg(test)]
fn tokenize(source: &str) -> Result<Vec<IrLine<'_>>, (IrError<'_>, usize)> {
    let lines = source
        .lines()
        .enumerate()
        .map(|(num, line)| (num + 1, line.trim()));

    let mut token_lines = Vec::new();
    let mut tokenization_data = TokenizationData::new();

    for (line_num, line) in lines {
        match tokenize_line_or_create_command(line, &mut tokenization_data) {
            Ok(tokens) => token_lines.push(tokens),
            Err(e) => return Err((e, line_num)),
        }
    }

    Ok(token_lines)
}

#[test]
fn tokenize_nothing() {
    assert_eq!(Ok(Vec::new()), tokenize(""));
}

#[test]
fn tokenize_phone() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a"))])]), tokenize("a"));
}

#[test]
fn tokenize_long_phone() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("abcdefg"))])]), tokenize("abcdefg"));
}

#[test]
fn tokenize_phones() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Phone(Phone::Symbol("bc")), IrToken::Phone(Phone::Symbol("def"))])]), tokenize("a bc def"));
}

#[test]
fn tokenize_lines_of_phones() {
    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Phone(Phone::Symbol("bc")), IrToken::Phone(Phone::Symbol("def"))]),
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("fed")), IrToken::Phone(Phone::Symbol("cb")), IrToken::Phone(Phone::Symbol("a"))]),
    ]), tokenize("a bc def\nfed cb a"));
}

#[test]
fn tokenize_lines_of_phones_and_nothing() {
    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Phone(Phone::Symbol("bc")), IrToken::Phone(Phone::Symbol("def"))]),
        IrLine::Empty,
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("fed")), IrToken::Phone(Phone::Symbol("cb")), IrToken::Phone(Phone::Symbol("a"))]),
    ]), tokenize("a bc def\n\nfed cb a"));
}

#[test]
fn tokenize_lines_of_phones_and_comment() {
    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Phone(Phone::Symbol("bc")), IrToken::Phone(Phone::Symbol("def"))]),
        IrLine::Empty,
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("fed")), IrToken::Phone(Phone::Symbol("cb")), IrToken::Phone(Phone::Symbol("a"))]),
    ]), tokenize("a bc def\n## this is a comment\nfed cb a"));
}

#[test]
fn tokenize_with_def() {
    assert_eq!(
        Ok(vec![IrLine::Empty, IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("b")), IrToken::Phone(Phone::Symbol("cd")), IrToken::Phone(Phone::Symbol("e"))])]),
        tokenize("DEFINE a b cd e\n@a")
    );
}

#[test]
fn tokenize_with_redef() {
    assert_eq!(Ok(vec![
        IrLine::Empty,
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("b")), IrToken::Phone(Phone::Symbol("cd")), IrToken::Phone(Phone::Symbol("e"))]),
        IrLine::Empty,
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("new")), IrToken::Phone(Phone::Symbol("content"))]),
    ]), tokenize("DEFINE a b cd e\n@a\nDEFINE a new content\n@a"));
}

#[test]
fn tokenize_empty_def() {
    assert_eq!(Err((IrError::EmptyDefinition, 1)), tokenize("DEFINE"));
}

#[test]
fn tokenize_late_def() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Phone(Phone::Symbol("DEFINE")),
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::Phone(Phone::Symbol("c")),
    ])]), tokenize("a DEFINE a b c"));
}

#[test]
fn tokenize_undef() {
    assert_eq!(Err((IrError::UndefinedDefinition("a"), 1)), tokenize("@a\nDEFINE a a b c"));
}

#[test]
fn tokenize_label() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Label("label")])]), tokenize("$label"));
}

#[test]
fn tokenize_phones_and_labels() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Label("label"),
        IrToken::Phone(Phone::Symbol("phone")),
        IrToken::Label("label_two"),
        IrToken::Phone(Phone::Symbol("b"))
    ])]), tokenize("a $label phone$label_two b"));
}

#[test]
fn tokenize_ltr() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Break(
        Break::Shift(Shift { dir: Direction::Ltr, kind: ShiftType::Stay })
    )])]), tokenize(">"));
}

#[test]
fn tokenize_double_ltr() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Break(
        Break::Shift(Shift { dir: Direction::Ltr, kind: ShiftType::Move })
    )])]), tokenize(">>"));
}

#[test]
fn tokenize_double_ltr_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Phone(Phone::Symbol("bc")), IrToken::Break(
        Break::Shift(Shift { dir: Direction::Ltr, kind: ShiftType::Move })
    ), IrToken::Phone(Phone::Symbol("de")), IrToken::Phone(Phone::Symbol("f"))])]), tokenize("a bc>>de f"));
}

#[test]
fn tokenize_rtl() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Break(
        Break::Shift(Shift { dir: Direction::Rtl, kind: ShiftType::Stay })
    )])]), tokenize("<"));
}

#[test]
fn tokenize_double_rtl() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Break(
        Break::Shift(Shift { dir: Direction::Rtl, kind: ShiftType::Move })
    )])]), tokenize("<<"));
}

#[test]
fn tokenize_double_rtl_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Phone(Phone::Symbol("bc")), IrToken::Break(
        Break::Shift(Shift { dir: Direction::Rtl, kind: ShiftType::Move })
    ), IrToken::Phone(Phone::Symbol("de")), IrToken::Phone(Phone::Symbol("f"))])]), tokenize("a bc<<de f"));
}

#[test]
fn tokenize_cond() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Break(Break::Cond)])]), tokenize("/"));
}

#[test]
fn tokenize_anti_cond() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Break(Break::AntiCond)])]), tokenize("//"));
}

#[test]
fn tokenize_cond_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Phone(Phone::Symbol("bc")),
        IrToken::Break(Break::Cond),
        IrToken::Phone(Phone::Symbol("de")),
        IrToken::Phone(Phone::Symbol("f"))
    ])]), tokenize("a bc/de f"));
}

#[test]
fn tokenize_anti_cond_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Phone(Phone::Symbol("bc")),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone(Phone::Symbol("de")),
        IrToken::Phone(Phone::Symbol("f"))
    ])]), tokenize("a bc//de f"));
}

#[test]
fn tokenize_scope_bounds() {
    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::ScopeStart(ScopeType::Optional)]),
    ]), tokenize("("));

    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::ScopeEnd(ScopeType::Optional)]),
    ]), tokenize(")"));

    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::ScopeStart(ScopeType::Selection)]),
    ]), tokenize("{"));

    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::ScopeEnd(ScopeType::Selection)]),
    ]), tokenize("}"));
}

#[test]
fn tokenize_scope_bounds_with_suroundings() {
    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::ScopeStart(ScopeType::Optional), IrToken::Phone(Phone::Symbol("b"))]),
    ]), tokenize("a(b"));

    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::ScopeEnd(ScopeType::Optional), IrToken::Phone(Phone::Symbol("b"))]),
    ]), tokenize("a)b"));

    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::ScopeStart(ScopeType::Selection), IrToken::Phone(Phone::Symbol("b"))]),
    ]), tokenize("a{b"));

    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::ScopeEnd(ScopeType::Selection), IrToken::Phone(Phone::Symbol("b"))]),
    ]), tokenize("a}b"));
}


#[test]
fn tokenize_gap() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Gap])]), tokenize(".."));
}

#[test]
fn tokenize_gap_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Gap, IrToken::Phone(Phone::Symbol("b")),])]), tokenize("a .. b"));
}

#[test]
fn tokenize_gap_with_close_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a..b"))])]), tokenize("a..b"));
}

#[test]
fn tokenize_any() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Any])]), tokenize("*"));
}

#[test]
fn tokenize_any_with_suroundings() {
    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Any, IrToken::Phone(Phone::Symbol("b"))]),
    ]), tokenize("a*b"));
}

#[test]
fn tokenize_sep() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::ArgSep])]), tokenize(","));
}

#[test]
fn tokenize_sep_with_suroundings() {
    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::ArgSep, IrToken::Phone(Phone::Symbol("b"))]),
    ]), tokenize("a,b"));
}

#[test]
fn tokenize_input() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::CondType(CondType::Pattern)])]), tokenize("_"))
}

#[test]
fn tokenize_input_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a")), IrToken::CondType(CondType::Pattern), IrToken::Phone(Phone::Symbol("b"))])]), tokenize("a _ b"))
}

#[test]
fn tokenize_input_with_contacting() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("a_b"))])]), tokenize("a_b"))
}

#[test]
fn print_statement() {
    assert_eq!(Ok(vec![IrLine::Cmd(Command::Print, "test message")]), tokenize("PRINT test message"))
}

#[test]
fn escape_print() {
    let shift_token = IrToken::Break(Break::Shift(Shift { dir: Direction::Ltr, kind: ShiftType::Move }));
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("\\PRINT")), shift_token, IrToken::Phone(Phone::Symbol("escaped"))])]), tokenize("\\PRINT >> escaped"))
}

#[test]
fn escape_definition_call() {
    let shift_token = IrToken::Break(Break::Shift(Shift { dir: Direction::Ltr, kind: ShiftType::Move }));
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone(Phone::Symbol("\\@a")), shift_token])]), tokenize("\\@a >>"))
}

#[test]
fn escape_escape() {
    assert_eq!(Err((IrError::UndefinedDefinition("a"), 1)), tokenize("\\\\@a >>"))
}

#[test]
fn tokenize_simple() {
    let tokens = tokenize("##this is a comment\nDEFINE V { i, e, a, u, o }\n\n$stops{p, t, k} >> $stops{b, d, g} / @V _ @V / _ {l, r} // h _");

    assert_eq!(Ok(vec![
        IrLine::Empty,
        IrLine::Empty,
        IrLine::Empty,
        IrLine::Ir(vec![
            IrToken::Label("stops"),
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone(Phone::Symbol("p")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("t")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("k")),
            IrToken::ScopeEnd(ScopeType::Selection),

            IrToken::Break(Break::Shift(Shift { dir: Direction::Ltr, kind: ShiftType::Move })),

            IrToken::Label("stops"),
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("d")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("g")),
            IrToken::ScopeEnd(ScopeType::Selection),

            IrToken::Break(Break::Cond),

            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone(Phone::Symbol("i")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("e")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("u")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("o")),
            IrToken::ScopeEnd(ScopeType::Selection),

            IrToken::CondType(CondType::Pattern),

            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone(Phone::Symbol("i")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("e")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("u")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("o")),
            IrToken::ScopeEnd(ScopeType::Selection),

            IrToken::Break(Break::Cond),
            IrToken::CondType(CondType::Pattern),

            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone(Phone::Symbol("l")),
            IrToken::ArgSep,
            IrToken::Phone(Phone::Symbol("r")),
            IrToken::ScopeEnd(ScopeType::Selection),

            IrToken::Break(Break::AntiCond),
            IrToken::Phone(Phone::Symbol("h")),
            IrToken::CondType(CondType::Pattern),
        ]),
    ]), tokens);
}