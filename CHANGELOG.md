# Changelog

## [Unreleased]

### Changed

- The `blocklet` example now builds against `marginalia` 0.4.0.
- Dependabot no longer bumps `dtolnay/rust-toolchain`, so the MSRV CI job stays pinned to
  `rust-version`.

## [0.2.1] - 2026-06-29

### Changed

- The `blocklet` example now builds against `marginalia` 0.2.0.
- CI and release workflows moved to `actions/checkout@v7`.

## [0.2.0] - 2026-06-10

### Added

- `OpenerRule` with two variants, set via `LayoutConfig::with_opener_rule`. `Always` (default) is
  the Haskell report behavior. `Conditional` opens a block only when the next token begins a new
  line at deeper indentation, so one-liners like `let x = e` stay flat (Koka style).
- Carry openers via `LayoutConfig::with_carry_openers`. A carry opener keeps its opener status
  across an immediately following bracket group on the same line, enabling trailing block arguments
  such as `each xs fn(x)` followed by an indented body.
- Integration test suite (`tests/prism.rs`) covering Eager mode, conditional openers, bracket
  suppression, and carry openers with exact token sequences.

### Changed

- **Breaking:** explicit braces are generalized to bracket groups. `with_explicit_braces` is now
  `with_brackets`, and the `is_explicit_open`/`is_explicit_close` fields are now
  `is_bracket_open`/`is_bracket_close`. Any number of bracket kinds (`( [ {`) can suppress layout
  while open.
- **Breaking:** virtual token spans moved. `v_sep` and `v_close` are now anchored at the end of the
  previous token instead of the start of the next one. `v_open` still anchors at the start of the
  first token of the block.
- `LayoutMode::Eager` keeps its top-level block open until end of input. A dedent below the first
  token's column no longer closes it.
- `LayoutConfig`'s `Debug` output reports `opener_rule`, `has_brackets`, and `has_carry_openers`.
- The `blocklet` example now builds against `marginalia` 0.1.4 (`TriviaKind` renamed to
  `BuiltinKind`).

## [0.1.1] - 2026-05-20

### Changed

- MSRV raised from 1.86 to 1.95.
- Releases publish with `cargo publish --locked`.
- CI moved to `actions/checkout@v6`, and Dependabot is enabled for Cargo and GitHub Actions.
- Removed the `justfile`. Run the example with
  `cargo run -p blocklet -- examples/blocklet/examples/sample.blocklet`.
- README wording fixes.

## [0.1.0] - 2026-05-20

### Added

- Initial release. `LayoutLexer` wraps any `(lo, T, hi)` token iterator and splices in virtual
  `v_open`, `v_sep`, and `v_close` tokens based on indentation, ready for a LALRPOP grammar.
- `Layout` trait for user token types, and `LayoutConfig` with a runtime opener predicate, explicit
  brace escape, and configurable tab width.
- `LayoutMode::Lazy` (layout after opener keywords, Haskell/ML style) and `LayoutMode::Eager`
  (top-level layout block, Python style).
- Composes with `marginalia`'s `TriviaLexer` for trivia preservation.
- `blocklet` example: a small block-let calculator using `let ... in` layout.

[Unreleased]: https://github.com/sdiehl/offsides/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/sdiehl/offsides/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/sdiehl/offsides/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/sdiehl/offsides/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/sdiehl/offsides/releases/tag/v0.1.0
