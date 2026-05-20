//! Tiny block-let calculator demonstrating offsides end-to-end.

use lalrpop_util::lalrpop_mod;
use marginalia::{TriviaLexer, TriviaTable};
use offsides::{LayoutConfig, LayoutLexer};
use thiserror::Error;

pub mod ast;
pub mod lexer;

lalrpop_mod!(
    #[allow(
        clippy::all,
        clippy::pedantic,
        clippy::unwrap_used,
        clippy::panic,
        unused_imports,
        dead_code,
        unreachable_pub,
        missing_debug_implementations
    )]
    parser
);

pub use parser::ProgramParser;

use crate::{
    ast::Expr,
    lexer::{is_opener, raw_lexer, LexicalError, Tok},
};

#[derive(Debug, Error)]
pub enum BlockletError {
    #[error("lex error: {0}")]
    Lex(#[from] LexicalError),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("eval error: {0}")]
    Eval(#[from] ast::EvalError),
}

#[must_use]
pub fn config() -> LayoutConfig<Tok> {
    LayoutConfig::new(is_opener)
}

pub fn parse(source: &str) -> Result<(Expr, TriviaTable), BlockletError> {
    let raw = raw_lexer(source);
    let trivia = TriviaLexer::new(raw, source);
    let mut layout = LayoutLexer::new(trivia, source, config());
    let program = ProgramParser::new()
        .parse(&mut layout)
        .map_err(|e| BlockletError::Parse(e.to_string()))?;
    let trivia = layout.into_inner();
    Ok((program, trivia.into_table()))
}

pub fn eval(source: &str) -> Result<i64, BlockletError> {
    let (expr, _) = parse(source)?;
    Ok(expr.eval()?)
}

/// Collect the stream of (kind, lo, hi) the parser would see, with `Tok`
/// printed via `Display`. Useful for snapshot tests.
#[must_use]
pub fn token_stream(source: &str) -> Vec<(String, usize, usize)> {
    let raw = raw_lexer(source);
    let trivia = TriviaLexer::new(raw, source);
    let layout = LayoutLexer::new(trivia, source, config());
    layout
        .filter_map(Result::ok)
        .map(|(lo, t, hi)| (t.to_string(), lo, hi))
        .collect()
}

#[cfg(test)]
mod tests;
