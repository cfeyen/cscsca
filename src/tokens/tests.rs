use super::*;

#[test]
fn tokenize_nothing() {
    assert_eq!(Ok(Vec::new()), tokenize(""));
}

#[test]
fn tokenize_phone() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone("a")])]), tokenize("a"));
}

#[test]
fn tokenize_long_phone() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone("abcdefg")])]), tokenize("abcdefg"));
}

#[test]
fn tokenize_phones() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone("a"), IrToken::Phone("bc"), IrToken::Phone("def")])]), tokenize("a bc def"));
}

#[test]
fn tokenize_lines_of_phones() {
    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone("a"), IrToken::Phone("bc"), IrToken::Phone("def")]),
        IrLine::Ir(vec![IrToken::Phone("fed"), IrToken::Phone("cb"), IrToken::Phone("a")]),
    ]), tokenize("a bc def\nfed cb a"));
}

#[test]
fn tokenize_lines_of_phones_and_nothing() {
    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone("a"), IrToken::Phone("bc"), IrToken::Phone("def")]),
        IrLine::Empty,
        IrLine::Ir(vec![IrToken::Phone("fed"), IrToken::Phone("cb"), IrToken::Phone("a")]),
    ]), tokenize("a bc def\n\nfed cb a"));
}

#[test]
fn tokenize_lines_of_phones_and_comment() {
    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone("a"), IrToken::Phone("bc"), IrToken::Phone("def")]),
        IrLine::Empty,
        IrLine::Ir(vec![IrToken::Phone("fed"), IrToken::Phone("cb"), IrToken::Phone("a")]),
    ]), tokenize("a bc def\n## this is a comment\nfed cb a"));
}

#[test]
fn tokenize_with_def() {
    assert_eq!(
        Ok(vec![IrLine::Empty, IrLine::Ir(vec![IrToken::Phone("b"), IrToken::Phone("cd"), IrToken::Phone("e")])]),
        tokenize("DEFINE a b cd e\n@a")
    );
}

#[test]
fn tokenize_with_redef() {
    assert_eq!(Ok(vec![
        IrLine::Empty,
        IrLine::Ir(vec![IrToken::Phone("b"), IrToken::Phone("cd"), IrToken::Phone("e")]),
        IrLine::Empty,
        IrLine::Ir(vec![IrToken::Phone("new"), IrToken::Phone("content")]),
    ]), tokenize("DEFINE a b cd e\n@a\nDEFINE a new content\n@a"));
}

#[test]
fn tokenize_empty_def() {
    assert_eq!(Err((IrError::EmptyDefinition, 1)), tokenize("DEFINE"));
}

#[test]
fn tokenize_late_def() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![
        IrToken::Phone("a"),
        IrToken::Phone("DEFINE"),
        IrToken::Phone("a"),
        IrToken::Phone("b"),
        IrToken::Phone("c"),
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
        IrToken::Phone("a"),
        IrToken::Label("label"),
        IrToken::Phone("phone"),
        IrToken::Label("label_two"),
        IrToken::Phone("b")
    ])]), tokenize("a $label phone$label_two b"));
}

#[test]
fn tokenize_ltr() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Break(
        Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Stay })
    )])]), tokenize(">"));
}

#[test]
fn tokenize_double_ltr() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Break(
        Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Move })
    )])]), tokenize(">>"));
}

#[test]
fn tokenize_double_ltr_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone("a"), IrToken::Phone("bc"), IrToken::Break(
        Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Move })
    ), IrToken::Phone("de"), IrToken::Phone("f")])]), tokenize("a bc>>de f"));
}

#[test]
fn tokenize_rtl() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Break(
        Break::Shift(Shift { dir: Direction::RTL, kind: ShiftType::Stay })
    )])]), tokenize("<"));
}

#[test]
fn tokenize_double_rtl() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Break(
        Break::Shift(Shift { dir: Direction::RTL, kind: ShiftType::Move })
    )])]), tokenize("<<"));
}

#[test]
fn tokenize_double_rtl_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone("a"), IrToken::Phone("bc"), IrToken::Break(
        Break::Shift(Shift { dir: Direction::RTL, kind: ShiftType::Move })
    ), IrToken::Phone("de"), IrToken::Phone("f")])]), tokenize("a bc<<de f"));
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
        IrToken::Phone("a"),
        IrToken::Phone("bc"),
        IrToken::Break(Break::Cond),
        IrToken::Phone("de"),
        IrToken::Phone("f")
    ])]), tokenize("a bc/de f"));
}

#[test]
fn tokenize_anti_cond_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![
        IrToken::Phone("a"),
        IrToken::Phone("bc"),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone("de"),
        IrToken::Phone("f")
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
        IrLine::Ir(vec![IrToken::Phone("a"), IrToken::ScopeStart(ScopeType::Optional), IrToken::Phone("b")]),
    ]), tokenize("a(b"));

    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone("a"), IrToken::ScopeEnd(ScopeType::Optional), IrToken::Phone("b")]),
    ]), tokenize("a)b"));

    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone("a"), IrToken::ScopeStart(ScopeType::Selection), IrToken::Phone("b")]),
    ]), tokenize("a{b"));

    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone("a"), IrToken::ScopeEnd(ScopeType::Selection), IrToken::Phone("b")]),
    ]), tokenize("a}b"));
}


#[test]
fn tokenize_gap() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Gap])]), tokenize(".."));
}

#[test]
fn tokenize_gap_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone("a"), IrToken::Gap, IrToken::Phone("b"),])]), tokenize("a .. b"));
}

#[test]
fn tokenize_gap_with_close_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone("a..b")])]), tokenize("a..b"));
}

#[test]
fn tokenize_any() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Any])]), tokenize("*"));
}

#[test]
fn tokenize_any_with_suroundings() {
    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone("a"), IrToken::Any, IrToken::Phone("b")]),
    ]), tokenize("a*b"));
}

#[test]
fn tokenize_sep() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::ArgSep])]), tokenize(","));
}

#[test]
fn tokenize_sep_with_suroundings() {
    assert_eq!(Ok(vec![
        IrLine::Ir(vec![IrToken::Phone("a"), IrToken::ArgSep, IrToken::Phone("b")]),
    ]), tokenize("a,b"));
}

#[test]
fn tokenize_input() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Input])]), tokenize("_"))
}

#[test]
fn tokenize_input_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone("a"), IrToken::Input, IrToken::Phone("b")])]), tokenize("a _ b"))
}

#[test]
fn tokenize_input_with_contacting() {
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone("a_b")])]), tokenize("a_b"))
}

#[test]
fn print_statement() {
    assert_eq!(Ok(vec![IrLine::Cmd(RuntimeCmd::Print, "test message")]), tokenize("PRINT test message"))
}

#[test]
fn escape_print() {
    let shift_token = IrToken::Break(Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Move }));
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone("\\PRINT"), shift_token, IrToken::Phone("escaped")])]), tokenize("\\PRINT >> escaped"))
}

#[test]
fn escape_definition_call() {
    let shift_token = IrToken::Break(Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Move }));
    assert_eq!(Ok(vec![IrLine::Ir(vec![IrToken::Phone("\\@a"), shift_token])]), tokenize("\\@a >>"))
}

#[test]
fn escape_escape() {
    assert_eq!(Err((IrError::UndefinedDefinition("a"), 1)), tokenize("\\\\@a >>"))
}

#[test]
fn tokenize_and_check_simple() {
    assert_eq!(Ok(vec![
        IrLine::Empty,
        IrLine::Empty,
        IrLine::Empty,
        IrLine::Ir(vec![
            IrToken::Label("stops"),
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone("p"),
            IrToken::ArgSep,
            IrToken::Phone("t"),
            IrToken::ArgSep,
            IrToken::Phone("k"),
            IrToken::ScopeEnd(ScopeType::Selection),

            IrToken::Break(Break::Shift(Shift { dir: Direction::LTR, kind: ShiftType::Move })),

            IrToken::Label("stops"),
            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone("b"),
            IrToken::ArgSep,
            IrToken::Phone("d"),
            IrToken::ArgSep,
            IrToken::Phone("g"),
            IrToken::ScopeEnd(ScopeType::Selection),

            IrToken::Break(Break::Cond),

            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone("i"),
            IrToken::ArgSep,
            IrToken::Phone("e"),
            IrToken::ArgSep,
            IrToken::Phone("a"),
            IrToken::ArgSep,
            IrToken::Phone("u"),
            IrToken::ArgSep,
            IrToken::Phone("o"),
            IrToken::ScopeEnd(ScopeType::Selection),

            IrToken::Input,

            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone("i"),
            IrToken::ArgSep,
            IrToken::Phone("e"),
            IrToken::ArgSep,
            IrToken::Phone("a"),
            IrToken::ArgSep,
            IrToken::Phone("u"),
            IrToken::ArgSep,
            IrToken::Phone("o"),
            IrToken::ScopeEnd(ScopeType::Selection),

            IrToken::Break(Break::Cond),
            IrToken::Input,

            IrToken::ScopeStart(ScopeType::Selection),
            IrToken::Phone("l"),
            IrToken::ArgSep,
            IrToken::Phone("r"),
            IrToken::ScopeEnd(ScopeType::Selection),

            IrToken::Break(Break::AntiCond),
            IrToken::Phone("h"),
            IrToken::Input,
        ]),
    ]), tokenize_and_check("##this is a comment\nDEFINE V { i, e, a, u, o }\n\n$stops{p, t, k} >> $stops{b, d, g} / @V _ @V / _ {l, r} // h _"))
}