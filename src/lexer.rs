use std::{collections::VecDeque, marker::PhantomData};

use crate::{
    column::ColumnIndex,
    layout::{Layout, LayoutConfig, LayoutMode, OpenerRule},
    stack::{CloseStep, IndentStack},
};

/// Iterator adapter that inserts virtual open/close/separator tokens into a
/// stream from a logos lexer (or any compatible token iterator), driven by an
/// off-side / indentation rule.
///
/// `LayoutLexer` wraps any `Iterator<Item = Result<(usize, T, usize), E>>` and
/// yields items of the same shape, with three kinds of virtual tokens spliced
/// in at zero-width spans:
///
/// - `T::v_open()` before the first token of a new layout block, anchored at
///   that token's start.
/// - `T::v_sep()` before each sibling token at the block's indent level,
///   anchored at the end of the previous token.
/// - `T::v_close()` after the last token of a layout block, anchored at the end
///   of that token.
///
/// The result can be passed straight into a lalrpop parser whose grammar names
/// those three virtual variants where it expects `{`, `;`, `}`.
///
/// See the `blocklet` example for a complete integration.
pub struct LayoutLexer<I, T, E>
where
    T: Layout,
{
    inner: I,
    source: String,
    cols: ColumnIndex,
    cfg: LayoutConfig<T>,
    stack: IndentStack,
    pending: VecDeque<Result<(usize, T, usize), E>>,
    pending_opener: bool,
    carrying: bool,
    bracket_depth: usize,
    prev_line: Option<usize>,
    last_hi: usize,
    done: bool,
    _marker: PhantomData<fn() -> Result<T, E>>,
}

impl<I, T, E> LayoutLexer<I, T, E>
where
    I: Iterator<Item = Result<(usize, T, usize), E>>,
    T: Layout,
{
    pub fn new(inner: I, source: impl Into<String>, cfg: LayoutConfig<T>) -> Self {
        let source = source.into();
        let cols = ColumnIndex::new(&source);
        Self {
            inner,
            source,
            cols,
            cfg,
            stack: IndentStack::default(),
            pending: VecDeque::new(),
            pending_opener: false,
            carrying: false,
            bracket_depth: 0,
            prev_line: None,
            last_hi: 0,
            done: false,
            _marker: PhantomData,
        }
    }

    /// Borrow the underlying iterator. Useful when the inner lexer carries
    /// state (such as `marginalia::TriviaLexer`'s trivia table) that must be
    /// recovered after parsing.
    pub fn inner(&self) -> &I {
        &self.inner
    }

    /// Mutable access to the underlying iterator.
    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.inner
    }

    /// Consume the layout lexer and return the underlying iterator.
    pub fn into_inner(self) -> I {
        self.inner
    }

    fn drain_eof(&mut self) {
        while self.stack.pop().is_some() {
            self.pending
                .push_back(Ok((self.last_hi, T::v_close(), self.last_hi)));
        }
    }

    fn handle_token(&mut self, lo: usize, tok: T, hi: usize) {
        let col = self.cols.column(&self.source, lo, self.cfg.tab_width);
        let cur_line = self.cols.line(lo);
        let new_line = self.prev_line.is_none_or(|pl| cur_line > pl);
        let in_brackets = self.bracket_depth > 0;
        let is_bopen = self.cfg.is_bracket_open.is_some_and(|f| f(&tok));
        let is_bclose = self.cfg.is_bracket_close.is_some_and(|f| f(&tok));

        if !in_brackets {
            let opens = match self.cfg.opener_rule {
                OpenerRule::Always => self.pending_opener,
                OpenerRule::Conditional => {
                    self.pending_opener && new_line && self.stack.top().is_none_or(|t| col > t)
                }
            };
            if opens || (self.prev_line.is_none() && self.cfg.mode == LayoutMode::Eager) {
                self.stack.push(col);
                self.pending.push_back(Ok((lo, T::v_open(), lo)));
            } else if new_line {
                let floor = usize::from(self.cfg.mode == LayoutMode::Eager);
                loop {
                    match self.stack.step(col, floor) {
                        CloseStep::Pop => {
                            self.stack.pop();
                            self.pending
                                .push_back(Ok((self.last_hi, T::v_close(), self.last_hi)));
                        }
                        CloseStep::Separator => {
                            self.pending
                                .push_back(Ok((self.last_hi, T::v_sep(), self.last_hi)));
                            break;
                        }
                        CloseStep::None => break,
                    }
                }
            }
            self.pending_opener = (self.cfg.is_opener)(&tok);
            self.carrying = self.cfg.is_carry_opener.is_some_and(|f| f(&tok))
                || (self.carrying && !new_line && is_bopen);
        }

        if is_bopen {
            self.bracket_depth += 1;
        } else if is_bclose {
            self.bracket_depth = self.bracket_depth.saturating_sub(1);
            if self.bracket_depth == 0 && self.carrying {
                self.pending_opener = true;
                self.carrying = false;
            }
        }

        self.pending.push_back(Ok((lo, tok, hi)));
        self.prev_line = Some(self.cols.line(hi.saturating_sub(1).max(lo)));
        self.last_hi = hi;
    }
}

impl<I, T, E> Iterator for LayoutLexer<I, T, E>
where
    I: Iterator<Item = Result<(usize, T, usize), E>>,
    T: Layout,
{
    type Item = Result<(usize, T, usize), E>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(item) = self.pending.pop_front() {
                return Some(item);
            }
            if self.done {
                return None;
            }
            match self.inner.next() {
                None => {
                    self.drain_eof();
                    self.done = true;
                }
                Some(Err(e)) => {
                    self.done = true;
                    return Some(Err(e));
                }
                Some(Ok((lo, tok, hi))) => self.handle_token(lo, tok, hi),
            }
        }
    }
}

impl<I, T, E> std::fmt::Debug for LayoutLexer<I, T, E>
where
    T: Layout,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutLexer")
            .field("stack_depth", &self.stack.depth())
            .field("bracket_depth", &self.bracket_depth)
            .field("pending", &self.pending.len())
            .finish_non_exhaustive()
    }
}
