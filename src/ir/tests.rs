
use crate::{executor::io_events::RuntimeIoEvent, ir::{tokenization_data::TokenizationData, tokenizer::tokenize_line_or_create_command, tokens::Break}, phones::Phone, rules::conditions::CondType, tokens::{Direction, ScopeType, Shift, ShiftType}, ONE};

use super::*;

/// Tokenizes rules, returns errors with the line they occur on
fn tokenize<'s>(rules: &'s str) -> Result<Vec<IrLine<'s>>, (IrError<'s>, usize)> {
    let mut lines = rules.lines().enumerate().map(|(line_num, line)| (line_num + 1, line));
    let mut ir = Vec::new();

    // Note: no IO can be preformed so no memory leaks occur if sources are not dropped
    let mut tokenization_data = TokenizationData::new();

    while let Some((line_num, line)) = lines.next() {
        let ir_line = tokenize_line_or_create_command(line, &mut (&mut lines).map(|(_, line)| line), &mut tokenization_data)
            .map_err(|e| (e, line_num))?;
        ir.push(ir_line);
    }

    Ok(ir)
}

#[test]
fn tokenize_nothing() {
    assert_eq!(Ok(Vec::new()), tokenize(""));
}

#[test]
fn tokenize_phone() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a"))], lines: ONE}]), tokenize("a"));
}

#[test]
fn tokenize_long_phone() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("abcdefg"))], lines: ONE}]), tokenize("abcdefg"));
}

#[test]
fn tokenize_phones() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Phone(Phone::Symbol("bc")), IrToken::Phone(Phone::Symbol("def"))], lines: ONE}]), tokenize("a bc def"));
}

#[test]
fn tokenize_lines_of_phones() {
    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Phone(Phone::Symbol("bc")), IrToken::Phone(Phone::Symbol("def"))], lines: ONE},
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("fed")), IrToken::Phone(Phone::Symbol("cb")), IrToken::Phone(Phone::Symbol("a"))], lines: ONE},
    ]), tokenize("a bc def\nfed cb a"));
}

#[test]
fn tokenize_lines_of_phones_and_nothing() {
    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Phone(Phone::Symbol("bc")), IrToken::Phone(Phone::Symbol("def"))], lines: ONE},
        IrLine::Empty,
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("fed")), IrToken::Phone(Phone::Symbol("cb")), IrToken::Phone(Phone::Symbol("a"))], lines: ONE},
    ]), tokenize("a bc def\n\nfed cb a"));
}

#[test]
fn tokenize_lines_of_phones_and_comment() {
    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Phone(Phone::Symbol("bc")), IrToken::Phone(Phone::Symbol("def"))], lines: ONE},
        IrLine::Empty,
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("fed")), IrToken::Phone(Phone::Symbol("cb")), IrToken::Phone(Phone::Symbol("a"))], lines: ONE},
    ]), tokenize("a bc def\n## this is a comment\nfed cb a"));
}

#[test]
fn tokenize_with_def() {
    assert_eq!(
        Ok(vec![IrLine::Empty, IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("b")), IrToken::Phone(Phone::Symbol("cd")), IrToken::Phone(Phone::Symbol("e"))], lines: ONE,}]),
        tokenize("DEFINE a b cd e\n@a")
    );
}

#[test]
fn tokenize_with_redef() {
    assert_eq!(Ok(vec![
        IrLine::Empty,
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("b")), IrToken::Phone(Phone::Symbol("cd")), IrToken::Phone(Phone::Symbol("e"))], lines: ONE},
        IrLine::Empty,
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("new")), IrToken::Phone(Phone::Symbol("content"))], lines: ONE},
    ]), tokenize("DEFINE a b cd e\n@a\nDEFINE a new content\n@a"));
}

#[test]
fn tokenize_empty_def() {
    assert_eq!(Err((IrError::EmptyDefinition, 1)), tokenize("DEFINE"));
}

#[test]
fn tokenize_late_def() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Phone(Phone::Symbol("DEFINE")),
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Phone(Phone::Symbol("b")),
        IrToken::Phone(Phone::Symbol("c")),
    ], lines: ONE}]), tokenize("a DEFINE a b c"));
}

#[test]
fn tokenize_undef() {
    assert_eq!(Err((IrError::UndefinedDefinition("a"), 1)), tokenize("@a\nDEFINE a a b c"));
}

#[test]
fn tokenize_label() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Label("label")], lines: ONE}]), tokenize("$label"));
}

#[test]
fn tokenize_phones_and_labels() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Label("label"),
        IrToken::Phone(Phone::Symbol("phone")),
        IrToken::Label("label_two"),
        IrToken::Phone(Phone::Symbol("b"))
    ], lines: ONE}]), tokenize("a $label phone$label_two b"));
}

#[test]
fn tokenize_ltr() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Break(
        Break::Shift(Shift { dir: Direction::Ltr, kind: ShiftType::Stay })
    )], lines: ONE}]), tokenize(">"));
}

#[test]
fn tokenize_double_ltr() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Break(
        Break::Shift(Shift { dir: Direction::Ltr, kind: ShiftType::Move })
    )], lines: ONE}]), tokenize(">>"));
}

#[test]
fn tokenize_double_ltr_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Phone(Phone::Symbol("bc")), IrToken::Break(
        Break::Shift(Shift { dir: Direction::Ltr, kind: ShiftType::Move })
    ), IrToken::Phone(Phone::Symbol("de")), IrToken::Phone(Phone::Symbol("f"))], lines: ONE}]), tokenize("a bc>>de f"));
}

#[test]
fn tokenize_rtl() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Break(
        Break::Shift(Shift { dir: Direction::Rtl, kind: ShiftType::Stay })
    )], lines: ONE}]), tokenize("<"));
}

#[test]
fn tokenize_double_rtl() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Break(
        Break::Shift(Shift { dir: Direction::Rtl, kind: ShiftType::Move })
    )], lines: ONE}]), tokenize("<<"));
}

#[test]
fn tokenize_double_rtl_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Phone(Phone::Symbol("bc")), IrToken::Break(
        Break::Shift(Shift { dir: Direction::Rtl, kind: ShiftType::Move })
    ), IrToken::Phone(Phone::Symbol("de")), IrToken::Phone(Phone::Symbol("f"))], lines: ONE}]), tokenize("a bc<<de f"));
}

#[test]
fn tokenize_cond() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Break(Break::Cond)], lines: ONE}]), tokenize("/"));
}

#[test]
fn tokenize_anti_cond() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Break(Break::AntiCond)], lines: ONE}]), tokenize("//"));
}

#[test]
fn tokenize_cond_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Phone(Phone::Symbol("bc")),
        IrToken::Break(Break::Cond),
        IrToken::Phone(Phone::Symbol("de")),
        IrToken::Phone(Phone::Symbol("f"))
    ], lines: ONE}]), tokenize("a bc/de f"));
}

#[test]
fn tokenize_anti_cond_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::Phone(Phone::Symbol("bc")),
        IrToken::Break(Break::AntiCond),
        IrToken::Phone(Phone::Symbol("de")),
        IrToken::Phone(Phone::Symbol("f"))
    ], lines: ONE}]), tokenize("a bc//de f"));
}

#[test]
fn tokenize_scope_bounds() {
    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::ScopeStart(ScopeType::Optional)], lines: ONE},
    ]), tokenize("("));

    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::ScopeEnd(ScopeType::Optional)], lines: ONE},
    ]), tokenize(")"));

    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::ScopeStart(ScopeType::Selection)], lines: ONE},
    ]), tokenize("{"));

    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::ScopeEnd(ScopeType::Selection)], lines: ONE},
    ]), tokenize("}"));
}

#[test]
fn tokenize_scope_bounds_with_suroundings() {
    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::ScopeStart(ScopeType::Optional), IrToken::Phone(Phone::Symbol("b"))], lines: ONE},
    ]), tokenize("a(b"));

    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::ScopeEnd(ScopeType::Optional), IrToken::Phone(Phone::Symbol("b"))], lines: ONE},
    ]), tokenize("a)b"));

    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::ScopeStart(ScopeType::Selection), IrToken::Phone(Phone::Symbol("b"))], lines: ONE}
    ]), tokenize("a{b"));

    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::ScopeEnd(ScopeType::Selection), IrToken::Phone(Phone::Symbol("b"))], lines: ONE},
    ]), tokenize("a}b"));
}


#[test]
fn tokenize_gap() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Gap], lines: ONE}]), tokenize(".."));
}

#[test]
fn tokenize_gap_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Gap, IrToken::Phone(Phone::Symbol("b")),], lines: ONE}]), tokenize("a .. b"));
}

#[test]
fn tokenize_dot_with_suroundings() {
    assert_eq!(
        tokenize("a..b"),
        Err((IrError::ReservedCharacter('.'), 1))
    );

    assert_eq!(
        tokenize("a.b"),
        Err((IrError::ReservedCharacter('.'), 1))
    );

    assert_eq!(
        tokenize("a\\.b"),
        Ok(vec![IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a\\.b"))], lines: ONE}])
    );
}

#[test]
fn tokenize_any() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Any], lines: ONE}]), tokenize("*"));
}

#[test]
fn tokenize_any_with_suroundings() {
    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Any, IrToken::Phone(Phone::Symbol("b"))], lines: ONE},
    ]), tokenize("a*b"));
}

#[test]
fn tokenize_sep() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::ArgSep], lines: ONE}]), tokenize(","));
}

#[test]
fn tokenize_sep_with_suroundings() {
    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::ArgSep, IrToken::Phone(Phone::Symbol("b"))], lines: ONE},
    ]), tokenize("a,b"));
}

#[test]
fn tokenize_input() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::CondType(CondType::Pattern)], lines: ONE}]), tokenize("_"))
}

#[test]
fn tokenize_input_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::CondType(CondType::Pattern), IrToken::Phone(Phone::Symbol("b"))], lines: ONE}]), tokenize("a _ b"))
}

#[test]
fn tokenize_input_with_contacting() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a_b"))], lines: ONE}]), tokenize("a_b"));
}

#[test]
fn print_statement() {
    assert_eq!(Ok(vec![IrLine::IoEvent(IoEvent::Runtime(RuntimeIoEvent::Print { msg: "test message" }))]), tokenize("PRINT test message"));
}

#[test]
fn escape() {
    assert_eq!(Err((IrError::BadEscape(Some('P')), 1)), tokenize("\\PRINT >> escaped"));
    assert!(tokenize("\\_ >> escaped").is_ok());
    assert_eq!(Err((IrError::BadEscape(Some('_')), 1)), tokenize("\\_0 >> escaped"));
}

#[test]
fn escape_definition_call() {
    let shift_token = IrToken::Break(Break::Shift(Shift { dir: Direction::Ltr, kind: ShiftType::Move }));
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("\\@a")), shift_token], lines: ONE}]), tokenize("\\@a >>"));
}

#[test]
fn escape_escape() {
    assert_eq!(Err((IrError::UndefinedDefinition("a"), 1)), tokenize("\\\\@a >>"));
}

#[test]
fn tokenize_multi_line() {
    assert_eq!(Ok(vec![
        IrLine::Ir {
            tokens: vec![
                IrToken::Phone(Phone::Symbol("h")),
                IrToken::Break(Break::Shift(Shift { dir: Direction::Ltr, kind: ShiftType::Move })),
                IrToken::Break(Break::Cond),
                IrToken::CondType(CondType::Pattern),
                IrToken::Phone(Phone::Bound),
            ],
            lines: const { NonZero::new(2).expect("2 ought to be nonzero") },
        }
    ]), tokenize("h >> \\\n / _ #"));
}

#[test]
fn error_in_multi_line() {
    assert_eq!(Err((IrError::UndefinedDefinition("a"), 1)), tokenize("h >> \\\n @a"));
    assert_eq!(Err((IrError::UndefinedDefinition("a"), 3)), tokenize("h >> \\\n \n @a"));
}

#[test]
fn tokenize_simple() {
    let tokens = tokenize("##this is a comment\nDEFINE V { i, e, a, u, o }\n\n$stops{p, t, k} >> $stops{b, d, g} / @V _ @V / _ {l, r} // h _");

    assert_eq!(Ok(vec![
        IrLine::Empty,
        IrLine::Empty,
        IrLine::Empty,
        IrLine::Ir { tokens: vec![
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
        ], lines: ONE},
    ]), tokens);
}