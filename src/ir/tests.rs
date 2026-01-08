
use crate::{
    ONE, executor::io_events::RuntimeIoEvent, ir::{tokenization_data::TokenizationData, tokens::Break}, lexer::Lexer, phones::Phone, tokens::{CondType, Direction, ScopeType, Shift, ShiftType}
};

use super::*;

/// Tokenizes rules, returns errors with the line they occur on
fn tokenize<'s>(rules: &'s str) -> Result<Vec<IrLine<'s>>, (IrError<'s>, usize)> {
    let mut ir = Vec::new();

    // Note: no IO can be preformed so no memory leaks occur if sources are not dropped
    let mut tokenization_data = TokenizationData::new();

    let mut sir = Lexer::lex(rules);

    let mut last_line = 0;

    while !sir.is_empty() {
        let ir_line = ir_line_from_sir(&mut sir, &mut tokenization_data, &mut Vec::new())
            .map_err(|(e, lines)| (e, last_line + lines.get()))?;
        ir.push(ir_line);
        last_line = sir.line();
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
        IrLine::Empty { lines: ONE },
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("fed")), IrToken::Phone(Phone::Symbol("cb")), IrToken::Phone(Phone::Symbol("a"))], lines: ONE},
    ]), tokenize("a bc def\n\nfed cb a"));
}

#[test]
fn tokenize_lines_of_phones_and_comment() {
    assert_eq!(Ok(vec![
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("a")), IrToken::Phone(Phone::Symbol("bc")), IrToken::Phone(Phone::Symbol("def"))], lines: ONE},
        IrLine::Empty { lines: ONE },
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("fed")), IrToken::Phone(Phone::Symbol("cb")), IrToken::Phone(Phone::Symbol("a"))], lines: ONE},
    ]), tokenize("a bc def\n## this is a comment\nfed cb a"));
}

#[test]
fn tokenize_with_def() {
    assert_eq!(
        Ok(vec![IrLine::Empty { lines: ONE }, IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("b")), IrToken::Phone(Phone::Symbol("cd")), IrToken::Phone(Phone::Symbol("e"))], lines: ONE,}]),
        tokenize("DEFINE a b cd e\n@a")
    );
}

#[test]
fn tokenize_with_redef() {
    assert_eq!(Ok(vec![
        IrLine::Empty { lines: ONE },
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("b")), IrToken::Phone(Phone::Symbol("cd")), IrToken::Phone(Phone::Symbol("e"))], lines: ONE},
        IrLine::Empty { lines: ONE },
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("new")), IrToken::Phone(Phone::Symbol("content"))], lines: ONE},
    ]), tokenize("DEFINE a b cd e\n@a\nDEFINE a new content\n@a"));
}

#[test]
fn tokenize_empty_def() {
    assert_eq!(Err((IrError::UnnamedDefinition, 1)), tokenize("DEFINE"));
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
fn tokenize_lazy_def() {
    assert_eq!(Ok(vec![
        IrLine::Empty { lines: ONE },
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("b")), IrToken::Phone(Phone::Symbol("c"))], lines: ONE }
    ]), tokenize("DEFINE_LAZY a b c\n@a"));
}

#[test]
fn tokenize_lazy_def_before_content_def() {
    assert_eq!(Ok(vec![
        IrLine::Empty { lines: ONE },
        IrLine::Empty { lines: ONE },
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("c"))], lines: ONE }
    ]), tokenize("DEFINE_LAZY a @b\nDEFINE b c\n@a"));
}



#[test]
fn tokenize_recursive_lazy_def() {
    assert_eq!(Err((IrError::RecursiveLazyDefiniton("a"), 3)), tokenize("DEFINE_LAZY a @b\nDEFINE_LAZY b @a\n@a"));
}

#[test]
fn tokenize_lazy_def_redef() {
    assert_eq!(Ok(vec![
        IrLine::Empty { lines: ONE },
        IrLine::Empty { lines: ONE },
        IrLine::Empty { lines: ONE },
        IrLine::Ir { tokens: vec![IrToken::Phone(Phone::Symbol("c"))], lines: ONE }
    ]), tokenize("DEFINE b z\nDEFINE_LAZY a @b\nDEFINE b c\n@a"));
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
fn tokenize_repetition() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![
        IrToken::ScopeStart(ScopeType::Repetition),
        IrToken::Any,
        IrToken::ScopeEnd(ScopeType::Repetition),
    ], lines: ONE}]), tokenize("[*]"));


    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![
        IrToken::ScopeStart(ScopeType::Repetition),
        IrToken::Any,
        IrToken::Negative,
        IrToken::Phone(Phone::Symbol("w")),
        IrToken::ScopeEnd(ScopeType::Repetition),
    ], lines: ONE}]), tokenize("[* ! w]"));
}

#[test]
fn tokenize_repetition_with_suroundings() {
    assert_eq!(Ok(vec![IrLine::Ir { tokens: vec![
        IrToken::Phone(Phone::Symbol("a")),
        IrToken::ScopeStart(ScopeType::Repetition),
        IrToken::Any,
        IrToken::ScopeEnd(ScopeType::Repetition),
        IrToken::Phone(Phone::Symbol("b")),
    ], lines: ONE}]), tokenize("a [*] b"));
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
    assert_eq!(Err((IrError::BadEscape(Some('_')), 1)), tokenize("0\\_ >> escaped"));
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
    assert_eq!(Err((IrError::UndefinedDefinition("a"), 2)), tokenize("h >> \\\n @a"));
    assert_eq!(Err((IrError::UndefinedDefinition("a"), 3)), tokenize("h >> \\\n \n @a"));
}

#[test]
fn tokenize_simple() {
    let tokens = tokenize("##this is a comment\nDEFINE V { i, e, a, u, o }\n\n$stops{p, t, k} >> $stops{b, d, g} / @V _ @V / _ {l, r} // h _");

    assert_eq!(Ok(vec![
        IrLine::Empty { lines: ONE },
        IrLine::Empty { lines: ONE },
        IrLine::Empty { lines: ONE },
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