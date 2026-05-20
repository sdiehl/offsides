# offsides

Layout-sensitive (off-side rule) lexer adapter for [logos](https://crates.io/crates/logos) +
[lalrpop](https://crates.io/crates/lalrpop) grammars.

`offsides` slots into the standard Rust parsing stack and splices virtual open/close/separator
tokens into the token stream based on indentation, so a downstream LALRPOP grammar can match `v{`,
`v;`, `v}` (or whatever the user names them) where it would otherwise want braces and semicolons.
Subsumes Haskell's `do`/`where`/`let`/`of` layout, Python's INDENT/DEDENT, and PureScript/Elm/Idris
off-side rules as configurations.

It is a sibling to [`marginalia`](https://crates.io/crates/marginalia), and the two compose. Stack
`TriviaLexer` (trivia preservation) inside `LayoutLexer` (layout insertion) and pass the combined
iterator straight to a LALRPOP parser.

The shape of an integration:

```rust,ignore
use offsides::{LayoutLexer, LayoutConfig, LayoutMode};
use marginalia::TriviaLexer;

let raw = my_logos_lexer(source);
let trivia = TriviaLexer::new(raw, source);
let layout = LayoutLexer::new(trivia, source, LayoutConfig::new(is_my_opener));
let program = MyParser::new().parse(layout)?;
```

The user's token type must implement `offsides::Layout` (three constructors: `v_open`, `v_close`,
`v_sep`). Which real tokens open a layout block is a runtime predicate passed into `LayoutConfig`,
so the same enum can drive different layout rules in different contexts.

- `LayoutMode::Lazy` (default): layout fires only after an opener keyword (in style of Haskell, ML,
  F#, PureScript, Elm, Idris).
- `LayoutMode::Eager`: top-level itself is a layout block (Python-style).

Tab width and explicit-brace escape (`{ ... }` that suppresses layout) are configurable on
`LayoutConfig`.

## Example

[`examples/blocklet`](examples/blocklet) is a ~150-line block-let calculator with one layout opener
(`let`). Bindings are indent-separated, and the block is closed by `in`:

```
let
    x = 10
    y = 20
    z = x + y
in
    z * 2
```

Run it end-to-end:

```bash
cargo run -p blocklet -- examples/blocklet/examples/sample.blocklet
```

## License

MIT. See [LICENSE](LICENSE).
