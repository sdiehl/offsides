use std::fmt;

use logos::{Lexer, Logos};
use marginalia::{BuiltinKind, Classify, TriviaPiece};
use offsides::Layout;
use thiserror::Error;

#[derive(Clone, Debug, Logos, PartialEq, Eq)]
#[logos(skip r"[ \t\f\r\n]+")]
pub enum Tok {
    #[token("let")]
    Let,
    #[token("in")]
    In,

    #[token("=")]
    Eq,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    #[regex(r"-?[0-9]+", |l| l.slice().parse::<i64>().ok())]
    Num(i64),

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |l| l.slice().to_owned(), priority = 2)]
    Ident(String),

    #[regex(r"--[^\n]*", |l| l.slice().to_owned(), allow_greedy = true)]
    LineComment(String),

    /// Virtual open-block, inserted by `offsides::LayoutLexer`.
    VOpen,
    /// Virtual close-block.
    VClose,
    /// Virtual separator.
    VSemi,
}

impl Layout for Tok {
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

impl Classify for Tok {
    fn trivia(&self) -> Option<TriviaPiece<'_>> {
        match self {
            Self::LineComment(s) => Some(TriviaPiece {
                kind: BuiltinKind::Line,
                text: s,
            }),
            _ => None,
        }
    }
}

#[must_use]
pub fn is_opener(tok: &Tok) -> bool {
    matches!(tok, Tok::Let)
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Let => f.write_str("let"),
            Self::In => f.write_str("in"),
            Self::Eq => f.write_str("="),
            Self::Plus => f.write_str("+"),
            Self::Minus => f.write_str("-"),
            Self::Star => f.write_str("*"),
            Self::Slash => f.write_str("/"),
            Self::LParen => f.write_str("("),
            Self::RParen => f.write_str(")"),
            Self::Num(n) => write!(f, "{n}"),
            Self::Ident(s) | Self::LineComment(s) => f.write_str(s),
            Self::VOpen => f.write_str("v{"),
            Self::VClose => f.write_str("v}"),
            Self::VSemi => f.write_str("v;"),
        }
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum LexicalError {
    #[error("invalid token at byte offset {0}")]
    InvalidToken(usize),
}

#[must_use]
pub fn raw_lexer(input: &str) -> RawLexer<'_> {
    RawLexer {
        inner: Tok::lexer(input),
    }
}

pub struct RawLexer<'input> {
    inner: Lexer<'input, Tok>,
}

impl Iterator for RawLexer<'_> {
    type Item = Result<(usize, Tok, usize), LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.inner.next()?;
        let span = self.inner.span();
        Some(match tok {
            Ok(t) => Ok((span.start, t, span.end)),
            Err(()) => Err(LexicalError::InvalidToken(span.start)),
        })
    }
}

impl fmt::Debug for RawLexer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawLexer").finish_non_exhaustive()
    }
}
