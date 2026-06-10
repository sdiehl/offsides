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
    /// the top-level indent, every later indent change emits virtual tokens,
    /// and the top-level block closes only at end of input.
    Eager,
}

/// When does the token after an opener actually start a block?
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OpenerRule {
    /// The next token always starts a block (Haskell report style).
    #[default]
    Always,
    /// The next token starts a block only if it begins a new line at deeper
    /// indentation than the enclosing block. Otherwise the opener is inert,
    /// so a one-liner like `let x = e` stays flat.
    Conditional,
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

    /// Does this token open a bracket group that suppresses layout? Typical:
    /// `( [ {`. While brackets are open no virtual tokens are emitted. `None`
    /// disables.
    pub is_bracket_open: Option<fn(&T) -> bool>,

    /// Does this token close a bracket group? Pairs with `is_bracket_open`.
    /// `None` disables.
    pub is_bracket_close: Option<fn(&T) -> bool>,

    /// Does this token keep its opener status across an immediately following
    /// bracket group on the same line? With `fn` in this set, `fn(x)` followed
    /// by an indented line opens a block after the closing paren, which gives
    /// trailing-block-argument syntax. `None` disables.
    pub is_carry_opener: Option<fn(&T) -> bool>,

    /// Off-side rule variant.
    pub mode: LayoutMode,

    /// Whether openers fire unconditionally or only on newline plus indent.
    pub opener_rule: OpenerRule,

    /// Columns per tab character. Defaults to `1`.
    pub tab_width: usize,

    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T: Layout> LayoutConfig<T> {
    /// Build a new config with the given opener predicate, no brackets, no
    /// carry openers, `Lazy` mode, `Always` openers, and `tab_width = 1`.
    #[must_use]
    pub fn new(is_opener: fn(&T) -> bool) -> Self {
        Self {
            is_opener,
            is_bracket_open: None,
            is_bracket_close: None,
            is_carry_opener: None,
            mode: LayoutMode::Lazy,
            opener_rule: OpenerRule::Always,
            tab_width: 1,
            _marker: std::marker::PhantomData,
        }
    }

    /// Enable bracket pairs that suppress layout while open.
    #[must_use]
    pub fn with_brackets(mut self, open: fn(&T) -> bool, close: fn(&T) -> bool) -> Self {
        self.is_bracket_open = Some(open);
        self.is_bracket_close = Some(close);
        self
    }

    /// Enable carry openers: tokens whose opener status survives an
    /// immediately following bracket group started on the same line.
    #[must_use]
    pub fn with_carry_openers(mut self, is_carry: fn(&T) -> bool) -> Self {
        self.is_carry_opener = Some(is_carry);
        self
    }

    /// Set the off-side rule variant.
    #[must_use]
    pub const fn with_mode(mut self, mode: LayoutMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set when openers fire.
    #[must_use]
    pub const fn with_opener_rule(mut self, rule: OpenerRule) -> Self {
        self.opener_rule = rule;
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
            .field("opener_rule", &self.opener_rule)
            .field("tab_width", &self.tab_width)
            .field("has_brackets", &self.is_bracket_open.is_some())
            .field("has_carry_openers", &self.is_carry_opener.is_some())
            .finish_non_exhaustive()
    }
}
