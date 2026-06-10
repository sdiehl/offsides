//! Integration tests for conditional-opener layout: Eager mode, conditional
//! openers, bracket suppression, and a carry opener for trailing `fn(x)`
//! block heads. Assertions are exact token sequences.

use offsides::{Layout, LayoutConfig, LayoutLexer, LayoutMode, OpenerRule};

#[derive(Clone, Debug, PartialEq, Eq)]
enum T {
    Eq,
    Then,
    Else,
    FatArrow,
    Of,
    With,
    Fn,
    Let,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Id(&'static str),
    VOpen,
    VClose,
    VSemi,
}

impl Layout for T {
    fn v_open() -> Self {
        Self::VOpen
    }
    fn v_close() -> Self {
        Self::VClose
    }
    fn v_sep() -> Self {
        Self::VSemi
    }
}

fn opens(t: &T) -> bool {
    matches!(
        t,
        T::Eq | T::Then | T::Else | T::FatArrow | T::Of | T::With | T::Fn
    )
}

fn bopen(t: &T) -> bool {
    matches!(t, T::LParen | T::LBracket | T::LBrace)
}

fn bclose(t: &T) -> bool {
    matches!(t, T::RParen | T::RBracket | T::RBrace)
}

fn carry(t: &T) -> bool {
    matches!(t, T::Fn)
}

fn cfg() -> LayoutConfig<T> {
    LayoutConfig::new(opens)
        .with_mode(LayoutMode::Eager)
        .with_opener_rule(OpenerRule::Conditional)
        .with_brackets(bopen, bclose)
        .with_carry_openers(carry)
}

fn word(w: &'static str) -> T {
    match w {
        "=" => T::Eq,
        "then" => T::Then,
        "else" => T::Else,
        "=>" => T::FatArrow,
        "of" => T::Of,
        "with" => T::With,
        "fn" => T::Fn,
        "let" => T::Let,
        "(" => T::LParen,
        ")" => T::RParen,
        "[" => T::LBracket,
        "]" => T::RBracket,
        "{" => T::LBrace,
        "}" => T::RBrace,
        "," => T::Comma,
        _ => T::Id(w),
    }
}

fn toks(src: &'static str) -> Vec<(usize, T, usize)> {
    let mut out = Vec::new();
    let mut start = None;
    for (i, c) in src.char_indices() {
        match (c.is_ascii_whitespace(), start) {
            (false, None) => start = Some(i),
            (true, Some(s)) => {
                out.push((s, word(&src[s..i]), i));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        out.push((s, word(&src[s..]), src.len()));
    }
    out
}

fn layout(src: &'static str) -> Vec<T> {
    let iter = toks(src).into_iter().map(Ok::<_, ()>);
    LayoutLexer::new(iter, src, cfg())
        .filter_map(Result::ok)
        .map(|(_, t, _)| t)
        .collect()
}

fn layout_spanned(src: &'static str) -> Vec<(usize, T, usize)> {
    let iter = toks(src).into_iter().map(Ok::<_, ()>);
    LayoutLexer::new(iter, src, cfg())
        .filter_map(Result::ok)
        .collect()
}

// tiny-prism's hand-rolled layout function, transcribed verbatim as a
// reference for differential testing.
fn reference(src: &str, tokens: Vec<(usize, T, usize)>) -> Vec<(usize, T, usize)> {
    let mut starts = vec![0usize];
    for (i, b) in src.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    let pos = |off: usize| {
        let line = starts.partition_point(|&s| s <= off) - 1;
        (line, off - starts[line])
    };
    let mut out = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut depth = 0usize;
    let mut prev_line = usize::MAX;
    let mut pending = false;
    let mut after_fn = false;
    let mut last = 0usize;
    for (lo, tok, hi) in tokens {
        let (line, col) = pos(lo);
        if depth == 0 {
            let open = match stack.last() {
                None => true,
                Some(&t) => pending && line != prev_line && col > t,
            };
            if open {
                stack.push(col);
                out.push((lo, T::VOpen, lo));
            } else if line != prev_line {
                while stack.len() > 1 && stack.last().is_some_and(|&t| col < t) {
                    stack.pop();
                    out.push((last, T::VClose, last));
                }
                if stack.last() == Some(&col) {
                    out.push((last, T::VSemi, last));
                }
            }
            pending = opens(&tok);
            after_fn = matches!(tok, T::Fn) || (after_fn && line == prev_line && tok == T::LParen);
        }
        if bopen(&tok) {
            depth += 1;
        } else if bclose(&tok) {
            depth = depth.saturating_sub(1);
            if depth == 0 && after_fn {
                pending = true;
                after_fn = false;
            }
        }
        prev_line = pos(hi.saturating_sub(1).max(lo)).0;
        out.push((lo, tok, hi));
        last = hi;
    }
    for _ in stack {
        out.push((last, T::VClose, last));
    }
    out
}

use T::{
    Comma, Eq as VEq, FatArrow, Id, LBracket, LParen, Let, Of, RBracket, RParen, Then, VClose,
    VOpen, VSemi, With,
};

#[test]
fn matches_tiny_prism_reference_algorithm() {
    let sources = [
        "main =\n  f x\n  g y",
        "main =\n  let x = 1\n  f x",
        "main =\n  match x of\n    Some y =>\n      f y\n    None => g",
        "main =\n  handle c with\n    op x k => k x\n    return v => v",
        "main =\n  if c then\n    a\n  else\n    b",
        "a =\n  b =\n    c\nd = e",
        "xs = [ 1 ,\n  2 ,\n  3 ]\nys = 0",
        "x =\n  ( a + b )\n  c",
        "each xs fn ( x )\n  f x\ndone",
        "  a\nb",
        "a =\n  b\nc",
        "f = fn ( g ( x ) )\n  body\nz",
        "a =\n    b\n  c",
        "x =\n  y =\n    z\n  w",
        "f = fn\n  x",
        "a ( b\n c ) d\ne",
        "x = [ a\n]\ny",
    ];
    for src in sources {
        assert_eq!(
            layout_spanned(src),
            reference(src, toks(src)),
            "src {src:?}"
        );
    }
}

#[test]
fn fn_body_block() {
    let src = "main =\n  f x\n  g y";
    assert_eq!(
        layout(src),
        vec![
            VOpen,
            Id("main"),
            VEq,
            VOpen,
            Id("f"),
            Id("x"),
            VSemi,
            Id("g"),
            Id("y"),
            VClose,
            VClose,
        ]
    );
}

#[test]
fn one_liner_let_does_not_open() {
    let src = "main =\n  let x = 1\n  f x";
    assert_eq!(
        layout(src),
        vec![
            VOpen,
            Id("main"),
            VEq,
            VOpen,
            Let,
            Id("x"),
            VEq,
            Id("1"),
            VSemi,
            Id("f"),
            Id("x"),
            VClose,
            VClose,
        ]
    );
}

#[test]
fn match_arms_with_fat_arrow_openers() {
    let src = "main =\n  match x of\n    Some y =>\n      f y\n    None => g";
    assert_eq!(
        layout(src),
        vec![
            VOpen,
            Id("main"),
            VEq,
            VOpen,
            Id("match"),
            Id("x"),
            Of,
            VOpen,
            Id("Some"),
            Id("y"),
            FatArrow,
            VOpen,
            Id("f"),
            Id("y"),
            VClose,
            VSemi,
            Id("None"),
            FatArrow,
            Id("g"),
            VClose,
            VClose,
            VClose,
        ]
    );
}

#[test]
fn handler_arms_after_with() {
    let src = "main =\n  handle c with\n    op x k => k x\n    return v => v";
    assert_eq!(
        layout(src),
        vec![
            VOpen,
            Id("main"),
            VEq,
            VOpen,
            Id("handle"),
            Id("c"),
            With,
            VOpen,
            Id("op"),
            Id("x"),
            Id("k"),
            FatArrow,
            Id("k"),
            Id("x"),
            VSemi,
            Id("return"),
            Id("v"),
            FatArrow,
            Id("v"),
            VClose,
            VClose,
            VClose,
        ]
    );
}

#[test]
fn if_then_else_blocks() {
    let src = "main =\n  if c then\n    a\n  else\n    b";
    assert_eq!(
        layout(src),
        vec![
            VOpen,
            Id("main"),
            VEq,
            VOpen,
            Id("if"),
            Id("c"),
            Then,
            VOpen,
            Id("a"),
            VClose,
            VSemi,
            T::Else,
            VOpen,
            Id("b"),
            VClose,
            VClose,
            VClose,
        ]
    );
}

#[test]
fn nested_blocks_close_together() {
    let src = "a =\n  b =\n    c\nd = e";
    assert_eq!(
        layout(src),
        vec![
            VOpen,
            Id("a"),
            VEq,
            VOpen,
            Id("b"),
            VEq,
            VOpen,
            Id("c"),
            VClose,
            VClose,
            VSemi,
            Id("d"),
            VEq,
            Id("e"),
            VClose,
        ]
    );
}

#[test]
fn brackets_suppress_layout() {
    let src = "xs = [ 1 ,\n  2 ,\n  3 ]\nys = 0";
    assert_eq!(
        layout(src),
        vec![
            VOpen,
            Id("xs"),
            VEq,
            LBracket,
            Id("1"),
            Comma,
            Id("2"),
            Comma,
            Id("3"),
            RBracket,
            VSemi,
            Id("ys"),
            VEq,
            Id("0"),
            VClose,
        ]
    );
}

#[test]
fn block_can_start_at_bracket_open() {
    let src = "x =\n  ( a + b )\n  c";
    assert_eq!(
        layout(src),
        vec![
            VOpen,
            Id("x"),
            VEq,
            VOpen,
            LParen,
            Id("a"),
            Id("+"),
            Id("b"),
            RParen,
            VSemi,
            Id("c"),
            VClose,
            VClose,
        ]
    );
}

#[test]
fn trailing_fn_head_carries_opener_across_parens() {
    let src = "each xs fn ( x )\n  f x\ndone";
    assert_eq!(
        layout(src),
        vec![
            VOpen,
            Id("each"),
            Id("xs"),
            T::Fn,
            LParen,
            Id("x"),
            RParen,
            VOpen,
            Id("f"),
            Id("x"),
            VClose,
            VSemi,
            Id("done"),
            VClose,
        ]
    );
}

#[test]
fn dedent_below_top_level_keeps_bottom_block() {
    let src = "  a\nb";
    assert_eq!(layout(src), vec![VOpen, Id("a"), Id("b"), VClose]);
}

#[test]
fn virtual_spans_anchor_like_prism() {
    let src = "a =\n  b\nc";
    assert_eq!(
        layout_spanned(src),
        vec![
            (0, VOpen, 0),
            (0, Id("a"), 1),
            (2, VEq, 3),
            (6, VOpen, 6),
            (6, Id("b"), 7),
            (7, VClose, 7),
            (7, VSemi, 7),
            (8, Id("c"), 9),
            (9, VClose, 9),
        ]
    );
}
