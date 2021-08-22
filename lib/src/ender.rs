//! Report on or fix line endings.
//!
//! To find out the line endings given a `Read` trait object use [`crate::ender::read_eol_info()`]:
//!
//! ```
//! use std::error::Error;
//! use std::fs::File;
//! use whitespace_rs::ender;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!   let mut reader = "abc\n\r\r\n".as_bytes();
//!   let eol_info = ender::read_eol_info(&mut reader)?;
//!
//!   println!("{:?}", eol_info);
//!   Ok(())
//! }
//! ```
//!
//! To normalize line endings given a `Read` trait object, create a `Write` trait object and
//! use [`crate::ender::write_new_eols()`]:
//!
//! ```
//! use std::error::Error;
//! use std::fs::File;
//! use whitespace_rs::ender;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!   let mut reader = "abc\n\r\r\n".as_bytes();
//!   let mut writer = Vec::new();
//!   let num_lines = ender::write_new_eols(&mut reader, &mut writer, ender::EndOfLine::Lf)?;
//!
//!   println!("{}", num_lines);
//!   Ok(())
//! }
//! ```

use std::error::Error;
use std::io::{Read, Write};
use utf8_decode::UnsafeDecoder;

#[derive(PartialEq, Debug, Clone, Copy)]
/// Types of line endings.
pub enum EndOfLine {
  /// Carriage return.
  Cr,
  /// Line feed.
  Lf,
  /// Carriage return and line feed.
  CrLf,
}

/// File line information.
#[derive(Debug, PartialEq)]
pub struct EolInfo {
  pub cr: usize,
  pub lf: usize,
  pub crlf: usize,
  pub num_lines: usize,
  pub num_endings: usize,
}

impl Eq for EolInfo {}

impl EolInfo {
  /// Get the most common end-of-line based on the info.
  pub fn get_common_eol(self: Self) -> EndOfLine {
    let mut n = self.lf;
    let mut eol = EndOfLine::Lf;

    if self.crlf > n {
      n = self.crlf;
      eol = EndOfLine::CrLf;
    }

    if self.cr > n {
      eol = EndOfLine::Cr;
    }

    eol
  }
}

/// Read end-of-line information for a file.
pub fn read_eol_info(reader: &mut dyn Read) -> Result<EolInfo, Box<dyn Error>> {
  let mut eol_info = EolInfo {
    cr: 0,
    lf: 0,
    crlf: 0,
    num_endings: 0,
    num_lines: 1,
  };
  let mut decoder = UnsafeDecoder::new(reader.bytes()).peekable();

  loop {
    let c;
    match decoder.next() {
      Some(value) => c = value?,
      None => break,
    };
    if c == '\r' {
      if matches!(decoder.peek(), Some(Ok(c)) if *c == '\n') {
        eol_info.crlf += 1;
        decoder.next();
      } else {
        eol_info.cr += 1;
      }

      eol_info.num_lines += 1;
    } else if c == '\n' {
      eol_info.lf += 1;
      eol_info.num_lines += 1;
    }
  }

  eol_info.num_endings =
    (eol_info.cr > 0) as usize + (eol_info.lf > 0) as usize + (eol_info.crlf > 0) as usize;

  Ok(eol_info)
}

/// Write input file out with new end-of-lines.
pub fn write_new_eols(
  reader: &mut dyn Read,
  writer: &mut dyn Write,
  new_eol: EndOfLine,
) -> Result<usize, Box<dyn Error>> {
  let mut num_lines = 1;
  let newline_chars = match new_eol {
    EndOfLine::Cr => "\r".as_bytes(),
    EndOfLine::Lf => "\n".as_bytes(),
    EndOfLine::CrLf => "\r\n".as_bytes(),
  };
  let mut decoder = UnsafeDecoder::new(reader.bytes()).peekable();
  let mut buf = [0u8; 4];

  loop {
    let c;

    match decoder.next() {
      Some(value) => c = value?,
      None => break,
    };
    if c == '\r' {
      if matches!(decoder.peek(), Some(Ok(c)) if *c == '\n') {
        decoder.next();
      }

      num_lines += 1;
      writer.write(newline_chars)?;
    } else if c == '\n' {
      num_lines += 1;
      writer.write(newline_chars)?;
    } else {
      writer.write(c.encode_utf8(&mut buf).as_bytes())?;
    }
  }
  writer.flush()?;

  Ok(num_lines)
}

#[cfg(test)]
#[path = "ender_tests.rs"]
mod tests;
