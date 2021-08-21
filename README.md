# Whitespace Tools

A Rust library and command line tools for fixing whitespace problems in text files.

- Report on line endings and optionally standardize them to CR, LF or CRLF
- Report on line beginnings and optionally standarize them to spaces or tabs.
- Handles files that are a mixture of different endings or beginnings
- Allows configuring the tab size on input and output

The following command line tools are included in this crate:

- `ender` reports and optionally normalizes or changes line endings in text files.
- `spacer` reports on and optionally normalizes whitespace at the beginning of lines.

## License

Whitespace Tools is distributed under the terms of the MIT License.

See LICENSE and COPYRIGHT for details.
