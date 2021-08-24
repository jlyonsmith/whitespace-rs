//! Report on or fix line endings.
//!
//! To find out the line endings given a [`Read`] trait object use [`read_eol_info()`]:
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
//! To normalize line endings given a [`Read`] trait object, create a [`Write`] trait object and
//! use [`write_new_eols()`]:
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

// {grcov-excl-start}
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
// {grcov-excl-end}

/// File line information.
#[derive(Debug, PartialEq)]
pub struct EolInfo {
  /// Number of lines that end in carriage return
  pub cr: usize,
  /// Number of lines that end in line feeds
  pub lf: usize,
  /// Number of lines that end in carriage return/line feed
  pub crlf: usize,
  /// Total number of lines in the file (includes lines with no ending)
  pub num_lines: usize,
}

impl Eq for EolInfo {}

impl EolInfo {
  /// Get the most common end-of-line based on the info.
  pub fn get_common_eol(self: &Self) -> EndOfLine {
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

  pub fn num_endings(self: &Self) -> usize {
    (self.cr > 0) as usize + (self.lf > 0) as usize + (self.crlf > 0) as usize
  }
}

/// Read end-of-line information for a file.
pub fn read_eol_info(reader: &mut dyn Read) -> Result<EolInfo, Box<dyn Error>> {
  let mut eol_info = EolInfo {
    cr: 0,
    lf: 0,
    crlf: 0,
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
mod tests {
  use super::*;

  #[test]
  fn test_read_eol_info_lf() {
    let eol_info = read_eol_info(&mut "\n".as_bytes()).unwrap();

    assert_eq!(
      eol_info,
      EolInfo {
        cr: 0,
        lf: 1,
        crlf: 0,
        num_lines: 2,
      }
    );
  }

  #[test]
  fn test_read_eol_info_cr() {
    let eol_info = read_eol_info(&mut "\r".as_bytes()).unwrap();

    assert_eq!(
      eol_info,
      EolInfo {
        cr: 1,
        lf: 0,
        crlf: 0,
        num_lines: 2,
      }
    );
  }

  #[test]
  fn test_read_eol_info_crlf() {
    let eol_info = read_eol_info(&mut "\r\n".as_bytes()).unwrap();

    assert_eq!(
      eol_info,
      EolInfo {
        cr: 0,
        lf: 0,
        crlf: 1,
        num_lines: 2,
      }
    );
  }

  #[test]
  fn test_read_eol_info_mixed1() {
    let eol_info = read_eol_info(&mut "\n\r\n\r".as_bytes()).unwrap();

    assert_eq!(
      eol_info,
      EolInfo {
        cr: 1,
        lf: 1,
        crlf: 1,
        num_lines: 4,
      }
    );
  }

  #[test]
  fn test_write_new_file() {
    let mut input = "abc\n\r\r\n".as_bytes();
    let mut output = Vec::new();
    let num_lines = write_new_eols(&mut input, &mut output, EndOfLine::CrLf).unwrap();

    assert_eq!(num_lines, 4);
    assert_eq!(String::from_utf8(output).unwrap(), "abc\r\n\r\n\r\n")
  }
}
