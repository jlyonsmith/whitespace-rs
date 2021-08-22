# Whitespace Tools (Rust Edition)

![Coverage](https://github.com/jlyonsmith/whitespace-rs/blob/main/badges/coverage.svg)]

A Rust package and command line tools for fixing whitespace problems in text files.

- Reports on end-of-lines.
- Standardize end-of-linen to CR, LF or CRLF
- Report on beginnings-of-lines.
- Standarize beginnings-of-lines them to spaces or tabs.
- Handles `Read` trait objects that are a mixture of different endings or beginnings
- Allows configuring the tab size on input and output

## Command Line

The following command line tools are included in this crate using the `cli` feature flag which is installed by default:

- `ender` - reports and optionally normalizes or changes line endings in text files. See `ender --help` for details.
- `spacer` - reports on and optionally normalizes whitespace at the beginning of lines. See `spacer --help` for details.

## License

Whitespace Tools is distributed under the terms of the [Unlicense](http://unlicense.org/) license.

See the file UNLICENSE for details.
