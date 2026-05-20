use crate::{eval, token_stream};

#[test]
fn evaluates_single_binding() {
    let src = "let\n    x = 10\nin\n    x + 5\n";
    assert_eq!(eval(src).expect("eval"), 15);
}

#[test]
fn evaluates_multiple_bindings() {
    let src = "let\n    x = 1\n    y = 2\n    z = x + y\nin\n    z * 10\n";
    assert_eq!(eval(src).expect("eval"), 30);
}

#[test]
fn nested_let() {
    let src = "\
let
    x = 1
in
    let
        y = 2
    in
        x + y
";
    assert_eq!(eval(src).expect("eval"), 3);
}

#[test]
fn comment_inside_block_is_invisible_to_layout() {
    let src = "\
let
    -- first binding
    x = 1
    y = 2
in
    x + y
";
    assert_eq!(eval(src).expect("eval"), 3);
}

#[test]
fn virtual_tokens_appear_at_block_boundaries() {
    let src = "let\n    x = 1\n    y = 2\nin\n    x + y\n";
    let toks: Vec<String> = token_stream(src).into_iter().map(|(s, _, _)| s).collect();
    assert_eq!(
        toks,
        vec!["let", "v{", "x", "=", "1", "v;", "y", "=", "2", "v}", "in", "x", "+", "y",]
    );
}

#[test]
fn eof_closes_open_blocks() {
    let src = "let\n    x = 1\n    y = 2";
    let toks: Vec<String> = token_stream(src).into_iter().map(|(s, _, _)| s).collect();
    assert_eq!(
        toks,
        vec!["let", "v{", "x", "=", "1", "v;", "y", "=", "2", "v}"]
    );
}
