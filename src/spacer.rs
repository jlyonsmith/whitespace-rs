//! Report on or fix beginning of line spacing
//!
//! To find out the line beginnings given a [`Read`] trait object use [`read_bol_info()`]:
//!
//! ```
//! use std::error::Error;
//! use std::fs::File;
//! use whitespace_rs::spacer;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!   let mut reader = "abc\n\r\r\n".as_bytes();
//!   let bol_info = spacer::read_bol_info(&mut reader)?;
//!
//!   println!("{:?}", bol_info);
//!   Ok(())
//! }
//! ```
//!
//! To normalize line beginnings given a [`Read`] trait object, create a [`Write`] trait object and use [`write_new_bols()`]:
//!
//! ```
//! use std::error::Error;
//! use std::fs::File;
//! use whitespace_rs::spacer;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!   let mut reader = "abc\n\r\r\n".as_bytes();
//!   let mut writer = Vec::new();
//!   let bol_info = spacer::write_new_bols(&mut reader, &mut writer, spacer::BeginningOfLine::Tabs(2, true))?;
//!
//!   println!("{:?}", bol_info);
//!   Ok(())
//! }
//! ```

use std::cmp::max;
use std::error::Error;
use std::io::{Read, Write};
use utf8_decode::UnsafeDecoder;

// {grcov-excl-start}
#[derive(Debug, PartialEq)]
/// Types of line beginnings
pub enum BeginningOfLine {
  /// Tabs (and spaces if not rounding down extra spaces)
  Tabs(usize, bool),
  /// Spaces
  Spaces(usize),
}
// {grcov-excl-end}

#[derive(Debug, PartialEq)]
/// Information about line beginnings in the file
pub struct BolInfo {
  /// Number of lines that have no whitespace at the beginning
  pub none: usize,
  /// Number of all space line beginnings
  pub spaces: usize,
  /// Number of all tab line beginnings
  pub tabs: usize,
  /// Number of mixed space/tab line beginnings
  pub mixed: usize,
}

impl Eq for BolInfo {}

impl BolInfo {
  /// Get the most common beginning of line type in the file
  pub fn get_common_bol(self: &Self, tab_size: usize, round_down: bool) -> BeginningOfLine {
    if self.tabs > self.spaces {
      BeginningOfLine::Tabs(tab_size, round_down)
    } else {
      BeginningOfLine::Spaces(tab_size)
    }
  }
}

/// Read beginning of line information
pub fn read_bol_info(reader: &mut dyn Read) -> Result<BolInfo, Box<dyn Error>> {
  let mut bol_info = BolInfo {
    none: 0,
    spaces: 0,
    tabs: 0,
    mixed: 0,
  };
  let mut decoder = UnsafeDecoder::new(reader.bytes()).peekable();
  let mut at_bol = true;
  let (mut num_spaces, mut num_tabs) = (0, 0);

  loop {
    let c;
    match decoder.next() {
      Some(value) => c = value?,
      None => break,
    };

    if at_bol {
      if c == ' ' {
        num_spaces += 1;
      } else if c == '\t' {
        num_tabs += 1;
      } else {
        if num_spaces == 0 && num_tabs == 0 {
          bol_info.none += 1;
        } else if num_spaces > 0 && num_tabs > 0 {
          bol_info.mixed += 1;
        } else if num_spaces > 0 {
          bol_info.spaces += 1;
        } else {
          bol_info.tabs += 1;
        }
        at_bol = false;
      }
    } else if c == '\n' {
      num_spaces = 0;
      num_tabs = 0;
      at_bol = true;
    }
  }

  Ok(bol_info)
}

/// Write input file out with new beginning-of-lines
pub fn write_new_bols(
  reader: &mut dyn Read,
  writer: &mut dyn Write,
  new_bol: BeginningOfLine,
) -> Result<BolInfo, Box<dyn Error>> {
  let (tab_size, round_down) = match new_bol {
    BeginningOfLine::Spaces(tab_size) => (max(1, tab_size), false),
    BeginningOfLine::Tabs(tab_size, round_down) => (max(1, tab_size), round_down),
  };
  let mut bol_info = BolInfo {
    none: 0,
    spaces: 0,
    tabs: 0,
    mixed: 0,
  };
  let mut decoder = UnsafeDecoder::new(reader.bytes()).peekable();
  let mut buf = [0u8; 4];
  let mut s = String::new();
  let mut at_bol = true;
  let untabify = |s: &str| -> String {
    let mut t = String::new();

    for c in s.chars() {
      if c == '\t' {
        t.push_str(&" ".repeat(tab_size - (t.len() % tab_size)));
      } else {
        t.push(c);
      }
    }

    t
  };
  let tabify = |s: &str| -> (_, _) {
    let mut num_spaces = 0;
    let mut t = String::new();

    for c in s.chars() {
      if c == ' ' {
        num_spaces += 1;
      }

      if num_spaces % tab_size == 0 {
        t.push('\t');
        num_spaces = 0
      }
    }

    if num_spaces > 0 {
      if !round_down {
        t.push_str(&" ".repeat(num_spaces));
      } else {
        num_spaces = 0;
      }
    }

    (t, num_spaces)
  };

  loop {
    let c;

    match decoder.next() {
      Some(value) => c = value?,
      None => break,
    };
    if at_bol {
      if c == ' ' || c == '\t' {
        s.push(c);
      } else {
        if s.len() == 0 {
          bol_info.none += 1
        } else {
          s = untabify(&s);

          if let BeginningOfLine::Tabs(_, _) = new_bol {
            let (t, num_spaces) = tabify(&s);

            s = t;
            if num_spaces > 0 {
              bol_info.mixed += 1;
            } else {
              bol_info.tabs += 1;
            }
          } else {
            bol_info.spaces += 1;
          }

          writer.write(s.as_bytes())?;
        }

        writer.write(c.encode_utf8(&mut buf).as_bytes())?;

        if c == '\n' {
          s.clear();
        } else {
          at_bol = false;
        }
      }
    } else {
      writer.write(c.encode_utf8(&mut buf).as_bytes())?;

      if c == '\n' {
        s.clear();
        at_bol = true;
      }
    }
  }
  writer.flush()?;

  Ok(bol_info)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_read_bol_info() {
    let bol_info = read_bol_info(&mut "a\n\tb\n  c\n \td\n".as_bytes()).unwrap();

    assert_eq!(
      bol_info,
      BolInfo {
        none: 1,
        spaces: 1,
        tabs: 1,
        mixed: 1,
      }
    );
  }

  #[test]
  fn test_write_new_file_tabs_round_down() {
    let mut input = "\na\n  b\n     c\n".as_bytes();
    let mut output = Vec::new();
    let bol_info = write_new_bols(&mut input, &mut output, BeginningOfLine::Tabs(2, true)).unwrap();

    assert_eq!(
      bol_info,
      BolInfo {
        none: 2,
        spaces: 0,
        tabs: 2,
        mixed: 0
      }
    );
    assert_eq!(String::from_utf8(output).unwrap(), "\na\n\tb\n\t\tc\n");
  }

  #[test]
  fn test_write_new_file_tabs_no_round_down() {
    let mut input = "\na\n  b\n     c\n".as_bytes();
    let mut output = Vec::new();
    let bol_info =
      write_new_bols(&mut input, &mut output, BeginningOfLine::Tabs(2, false)).unwrap();

    assert_eq!(
      bol_info,
      BolInfo {
        none: 2,
        spaces: 0,
        tabs: 1,
        mixed: 1
      }
    );
    assert_eq!(String::from_utf8(output).unwrap(), "\na\n\tb\n\t\t c\n");
  }

  #[test]
  fn test_write_new_file_spaces() {
    let mut input = "\ta\n \t x\n\t\t\n".as_bytes();
    let mut output = Vec::new();
    let bol_info = write_new_bols(&mut input, &mut output, BeginningOfLine::Spaces(2)).unwrap();

    assert_eq!(
      bol_info,
      BolInfo {
        none: 0,
        spaces: 3,
        tabs: 0,
        mixed: 0
      }
    );
    assert_eq!(String::from_utf8(output).unwrap(), "  a\n   x\n    \n");
  }
}
