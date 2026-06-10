//! Layout-sensitive (off-side rule) lexer adapter for `logos` + `lalrpop`
//! pipelines.
//!
//! [`LayoutLexer`] wraps any `Iterator<Item = Result<(usize, T, usize), E>>`
//! and splices virtual open/close/separator tokens into the stream based on
//! the indentation of the input. The downstream parser sees a clean
//! brace-and-semicolon shape and never has to think about columns.
//!
//! ```ignore
//! use offsides::{LayoutLexer, LayoutConfig, LayoutMode, OpenerRule};
//!
//! let cfg = LayoutConfig::new(is_layout_opener)
//!     .with_mode(LayoutMode::Eager)
//!     .with_opener_rule(OpenerRule::Conditional)
//!     .with_brackets(is_bracket_open, is_bracket_close);
//! let layout = LayoutLexer::new(my_logos_lexer, source, cfg);
//! let ast = MyParser::new().parse(layout)?;
//! ```
//!
//! Composes with `marginalia::TriviaLexer`; see the `blocklet` example.

mod column;
mod lexer;
mod stack;

pub mod layout;

pub use layout::{Layout, LayoutConfig, LayoutMode, OpenerRule};
pub use lexer::LayoutLexer;
