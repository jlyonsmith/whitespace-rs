# Whitespace Tools (Rust Edition)

These are a couple of command line tools for fixing whitespace issues in files:

- `ender` reports and optionally normalizes or changes line endings in text files.
- `spacer` reports on and optionally normalizes or changes beginning of line (before the first character) whitespace.

## Installing the Rust Code Coverage Tools

To install Rust's native LLVM based code coverage tools:

```zsh
rustup install nightly
rustup default nightly
rustup component add llvm-tools-preview
cargo install grcov
```

The run `./get-coverage.sh` to generate a `.profraw` file in the `/scratch` directory, and HTML coverage report. Open it with `open ./target/debug/coverage/*.html`.
