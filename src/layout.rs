//! Public configuration types for layout-sensitive lexing.

/// Token types that participate in layout must implement `Layout` so the lexer
/// can construct virtual open/close/separator tokens to splice into the stream.
///
/// The user's token enum picks names for these three variants (e.g.
/// `VOpen`/`VClose`/`VSemi`, or `Indent`/`Dedent`/`Newline`). Only the three
/// constructor methods are required; classification of which *real* tokens
/// open a block is configured separately via [`LayoutConfig`].
pub trait Layout: Sized + 'static {
    /// Construct a virtual open-block token.
    fn v_open() -> Self;

    /// Construct a virtual close-block token.
    fn v_close() -> Self;

    /// Construct a virtual separator (semicolon-equivalent) between siblings
    /// inside the same layout block.
    fn v_sep() -> Self;
}

/// Off-side rule variant.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LayoutMode {
    /// Haskell-style: a layout block only opens after a keyword identified by
    /// [`LayoutConfig::is_opener`]. Top-level tokens have no implicit block.
    #[default]
    Lazy,
    /// Python-style: the whole file is one layout block. The first token sets
    /// the top-level indent and every later indent change emits virtual
    /// tokens.
    Eager,
}

/// Configuration for [`crate::LayoutLexer`].
///
/// Predicates are plain function pointers so that no allocation or `Box<dyn
/// Fn>` is needed; users pass an `fn item` such as `is_my_opener` defined
/// elsewhere.
#[derive(Clone)]
pub struct LayoutConfig<T: Layout> {
    /// Does this token start a layout block? The *next* token's column becomes
    /// the new block's indent level. For Haskell: `do | where | let | of`.
    pub is_opener: fn(&T) -> bool,

    /// Does this token explicitly suppress layout? Useful for languages that
    /// allow a literal `{` to escape the off-side rule. `None` disables.
    pub is_explicit_open: Option<fn(&T) -> bool>,

    /// Does this token resume layout after a suppression? Pairs with
    /// `is_explicit_open`. `None` disables.
    pub is_explicit_close: Option<fn(&T) -> bool>,

    /// Off-side rule variant.
    pub mode: LayoutMode,

    /// Columns per tab character. Defaults to `1`.
    pub tab_width: usize,

    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T: Layout> LayoutConfig<T> {
    /// Build a new config with the given opener predicate, no explicit braces,
    /// `Lazy` mode, and `tab_width = 1`.
    #[must_use]
    pub fn new(is_opener: fn(&T) -> bool) -> Self {
        Self {
            is_opener,
            is_explicit_open: None,
            is_explicit_close: None,
            mode: LayoutMode::Lazy,
            tab_width: 1,
            _marker: std::marker::PhantomData,
        }
    }

    /// Enable explicit brace pairs that suppress and resume layout.
    #[must_use]
    pub fn with_explicit_braces(mut self, open: fn(&T) -> bool, close: fn(&T) -> bool) -> Self {
        self.is_explicit_open = Some(open);
        self.is_explicit_close = Some(close);
        self
    }

    /// Set the off-side rule variant.
    #[must_use]
    pub const fn with_mode(mut self, mode: LayoutMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the tab-to-column expansion.
    #[must_use]
    pub const fn with_tab_width(mut self, tab_width: usize) -> Self {
        self.tab_width = tab_width;
        self
    }
}

impl<T: Layout> std::fmt::Debug for LayoutConfig<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutConfig")
            .field("mode", &self.mode)
            .field("tab_width", &self.tab_width)
            .field("has_explicit_open", &self.is_explicit_open.is_some())
            .field("has_explicit_close", &self.is_explicit_close.is_some())
            .finish_non_exhaustive()
    }
}
