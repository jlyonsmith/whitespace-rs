//! Report on or fix beginning of line spacing

use std::error::Error;
use std::io::{Read, Write};
use utf8_decode::UnsafeDecoder;

#[derive(Debug, PartialEq)]
/// Types of line beginnings
pub enum BeginningOfLine {
  /// Tabs and spaces if not rounding down extra spaces
  Tabs,
  /// Spaces
  Spaces,
}

#[derive(Debug, PartialEq)]
/// Information about line beginnings in the file
pub struct BolInfo {
  /// Number of spaces in line beginnings
  pub spaces: usize,
  /// Numbef of tabs in line beginnings
  pub tabs: usize,
}

impl Eq for BolInfo {}

impl BolInfo {
  /// Get the most common beginning of line type in the file
  pub fn get_common_bol(self: Self) -> BeginningOfLine {
    if self.tabs > self.spaces {
      BeginningOfLine::Tabs
    } else {
      BeginningOfLine::Spaces
    }
  }
}

/// Read beginning of line information
pub fn read_bol_info(reader: &mut dyn Read) -> Result<BolInfo, Box<dyn Error>> {
  let mut bol_info = BolInfo { spaces: 0, tabs: 0 };
  let mut decoder = UnsafeDecoder::new(reader.bytes()).peekable();
  let mut at_bol = true;

  loop {
    let c;
    match decoder.next() {
      Some(value) => c = value?,
      None => break,
    };

    if at_bol {
      if c == ' ' {
        bol_info.spaces += 1;
      } else if c == '\t' {
        bol_info.tabs += 1;
      } else {
        at_bol = false;
      }
    } else if c == '\n' {
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
  old_tab_size: usize,
  new_tab_size: usize,
  round_down: bool,
) -> Result<BolInfo, Box<dyn Error>> {
  let old_tab_size = std::cmp::max(1, old_tab_size);
  let new_tab_size = std::cmp::max(1, new_tab_size);
  let mut bol_info = BolInfo { spaces: 0, tabs: 0 };
  let mut decoder = UnsafeDecoder::new(reader.bytes()).peekable();
  let mut buf = [0u8; 4];
  let mut s = String::new();
  let mut at_bol = true;
  let untabify = |s: &str| -> String {
    let mut t = String::new();

    for c in s.chars() {
      if c == '\t' {
        t.push_str(&" ".repeat(new_tab_size - (t.len() % new_tab_size)));
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

      if num_spaces % old_tab_size == 0 {
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
        s = untabify(&s);

        if new_bol == BeginningOfLine::Tabs {
          let (t, num_spaces) = tabify(&s);

          s = t;
          bol_info.tabs += s.len() - num_spaces;
          bol_info.spaces += num_spaces;
        } else {
          bol_info.spaces += s.len();
        }

        writer.write(s.as_bytes())?;
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
    let bol_info = read_bol_info(&mut "  \txyz\n".as_bytes()).unwrap();

    assert_eq!(bol_info, BolInfo { spaces: 2, tabs: 1 });
  }

  #[test]
  fn test_write_new_file_round_down() {
    let mut input = " a\n  x\n    \n".as_bytes();
    let mut output = Vec::new();
    let bol_info =
      write_new_bols(&mut input, &mut output, BeginningOfLine::Tabs, 2, 4, true).unwrap();

    assert_eq!(bol_info, BolInfo { spaces: 0, tabs: 3 });
    assert_eq!(String::from_utf8(output).unwrap(), "a\n\tx\n\t\t\n");
  }

  #[test]
  fn test_write_new_file() {
    let mut input = " a\n   x\n    \n".as_bytes();
    let mut output = Vec::new();
    let bol_info =
      write_new_bols(&mut input, &mut output, BeginningOfLine::Tabs, 2, 2, false).unwrap();

    assert_eq!(bol_info, BolInfo { spaces: 2, tabs: 3 });
    assert_eq!(String::from_utf8(output).unwrap(), " a\n\t x\n\t\t\n");
  }

  #[test]
  fn test_write_new_file_tabs() {
    let mut input = "\ta\n \t x\n\t\t\n".as_bytes();
    let mut output = Vec::new();
    let bol_info =
      write_new_bols(&mut input, &mut output, BeginningOfLine::Spaces, 2, 2, true).unwrap();

    assert_eq!(bol_info, BolInfo { spaces: 9, tabs: 0 });
    assert_eq!(String::from_utf8(output).unwrap(), "  a\n   x\n    \n");
  }
}
