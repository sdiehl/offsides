default: test

build:
    cargo build --workspace --all-targets

test:
    cargo test --workspace --all-targets

fmt:
    cargo fmt --all
    dprint fmt

lint:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -Dwarnings

docs:
    cargo doc --workspace --no-deps --open

blocklet *ARGS:
    cargo run -p blocklet -- {{ARGS}}

release-dry-run:
    cargo publish --dry-run

clean:
    cargo clean
