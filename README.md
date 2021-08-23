# Whitespace Tools (Rust Edition)

[![Crates.io](https://img.shields.io/crates/v/whitespace-rs.svg)](https://crates.io/crates/whitespace-rs)
[![Docs.rs](https://docs.rs/whitespace-rs/badge.svg)](https://docs.rs/whitespace-rs)

A Rust package and command line tools for fixing whitespace problems in text files.

- Reports on end-of-lines.
- Standardize end-of-lines to CR, LF or CRLF
- Report on beginnings-of-lines.
- Standarize beginnings-of-lines to spaces or tabs.
- Handles a mixture of different endings or beginnings
- Allows configuring the tab size on both input and output

## Command Line

The command line tools `ender` and `spacer` are included in this crate using the `cli` feature flag (installed by default.)

- `ender` - reports and optionally normalizes or changes line endings in text files. See `ender --help` for details.
- `spacer` - reports on and optionally normalizes whitespace at the beginning of lines. See `spacer --help` for details.

## License

Whitespace Tools is distributed under the terms of the [Unlicense](http://unlicense.org/) license. See the file [`UNLICENSE`](UNLICENSE) for details.
