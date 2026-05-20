use std::{collections::VecDeque, marker::PhantomData};

use crate::{
    column::ColumnIndex,
    layout::{Layout, LayoutConfig, LayoutMode},
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
/// - `T::v_open()` before the first token of a new layout block.
/// - `T::v_sep()` before each sibling token at the block's indent level.
/// - `T::v_close()` after the last token of a layout block.
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
    explicit_depth: usize,
    prev_line: Option<usize>,
    last_hi: usize,
    started: bool,
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
            explicit_depth: 0,
            prev_line: None,
            last_hi: 0,
            started: false,
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

        let is_opener = (self.cfg.is_opener)(&tok);
        let is_xopen = self.cfg.is_explicit_open.is_some_and(|f| f(&tok));
        let is_xclose = self.cfg.is_explicit_close.is_some_and(|f| f(&tok));

        let mut just_opened = false;
        if self.pending_opener && self.explicit_depth == 0 {
            self.stack.push(col);
            self.pending.push_back(Ok((lo, T::v_open(), lo)));
            self.pending_opener = false;
            just_opened = true;
        } else if !self.started && self.cfg.mode == LayoutMode::Eager && self.explicit_depth == 0 {
            self.stack.push(col);
            self.pending.push_back(Ok((lo, T::v_open(), lo)));
            just_opened = true;
        }
        self.started = true;

        if !just_opened {
            if let Some(pl) = self.prev_line {
                if cur_line > pl && self.explicit_depth == 0 {
                    loop {
                        match self.stack.step(col) {
                            CloseStep::Pop => {
                                self.stack.pop();
                                self.pending.push_back(Ok((lo, T::v_close(), lo)));
                            }
                            CloseStep::Separator => {
                                self.pending.push_back(Ok((lo, T::v_sep(), lo)));
                                break;
                            }
                            CloseStep::None => break,
                        }
                    }
                }
            }
        }

        if is_xopen {
            self.explicit_depth += 1;
        }
        self.pending.push_back(Ok((lo, tok, hi)));
        if is_xclose {
            self.explicit_depth = self.explicit_depth.saturating_sub(1);
        }

        if is_opener && self.explicit_depth == 0 {
            self.pending_opener = true;
        }

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
            .field("explicit_depth", &self.explicit_depth)
            .field("pending", &self.pending.len())
            .finish_non_exhaustive()
    }
}
