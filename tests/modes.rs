//! Direct unit tests for the `LayoutLexer` over a hand-rolled token stream,
//! independent of `logos`. Verifies the algorithm in isolation.

use offsides::{Layout, LayoutConfig, LayoutLexer, LayoutMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum T {
    Do,
    Word,
    VOpen,
    VClose,
    VSep,
}

impl Layout for T {
    fn v_open() -> Self {
        Self::VOpen
    }
    fn v_close() -> Self {
        Self::VClose
    }
    fn v_sep() -> Self {
        Self::VSep
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_do(t: &T) -> bool {
    matches!(t, T::Do)
}

fn run(source: &'static str, tokens: Vec<(usize, T, usize)>, mode: LayoutMode) -> Vec<T> {
    let cfg = LayoutConfig::new(is_do).with_mode(mode);
    let iter = tokens
        .into_iter()
        .map::<Result<(usize, T, usize), ()>, _>(Ok);
    LayoutLexer::new(iter, source, cfg)
        .filter_map(Result::ok)
        .map(|(_, t, _)| t)
        .collect()
}

#[test]
fn lazy_mode_no_opener_means_no_layout() {
    // No `do` keyword, Lazy mode -> stream passes through unchanged.
    let src = "a\nb\nc";
    let toks = vec![(0, T::Word, 1), (2, T::Word, 3), (4, T::Word, 5)];
    let out = run(src, toks, LayoutMode::Lazy);
    assert_eq!(out, vec![T::Word, T::Word, T::Word]);
}

#[test]
fn eager_mode_wraps_top_level_in_a_block() {
    // Same input under Eager mode: top-level is a layout block, so VOpen
    // appears before the first token and VSep between same-column siblings.
    let src = "a\nb\nc";
    let toks = vec![(0, T::Word, 1), (2, T::Word, 3), (4, T::Word, 5)];
    let out = run(src, toks, LayoutMode::Eager);
    assert_eq!(
        out,
        vec![
            T::VOpen,
            T::Word,
            T::VSep,
            T::Word,
            T::VSep,
            T::Word,
            T::VClose,
        ]
    );
}

#[test]
fn dedent_closes_blocks_to_outer_level() {
    // do
    //   a
    //   b
    // c
    // Expect: do v{ a v; b v} c
    let src = "do\n  a\n  b\nc";
    let toks = vec![
        (0, T::Do, 2),
        (5, T::Word, 6),
        (9, T::Word, 10),
        (11, T::Word, 12),
    ];
    let out = run(src, toks, LayoutMode::Lazy);
    assert_eq!(
        out,
        vec![
            T::Do,
            T::VOpen,
            T::Word,
            T::VSep,
            T::Word,
            T::VClose,
            T::Word,
        ]
    );
}

#[test]
fn nested_blocks_close_in_order_at_eof() {
    // do
    //   do
    //     a
    // (eof) -> two v} in sequence
    let src = "do\n  do\n    a";
    let toks = vec![(0, T::Do, 2), (5, T::Do, 7), (12, T::Word, 13)];
    let out = run(src, toks, LayoutMode::Lazy);
    assert_eq!(
        out,
        vec![
            T::Do,
            T::VOpen,
            T::Do,
            T::VOpen,
            T::Word,
            T::VClose,
            T::VClose,
        ]
    );
}
