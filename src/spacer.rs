use clap::arg_enum;
use std::error::Error;
use std::io::{Read, Write};
use utf8_decode::UnsafeDecoder;

arg_enum! {
  #[derive(PartialEq, Debug, Clone, Copy)]
  pub enum BeginningOfLine {
      Tabs,
      Spaces,
      Auto,
  }
}

#[derive(Debug, PartialEq)]
pub struct LineInfo {
  pub spaces: usize,
  pub tabs: usize,
}

impl Eq for LineInfo {}

pub fn read_bol_info(reader: &mut dyn Read) -> Result<LineInfo, Box<dyn Error>> {
  let mut line_info = LineInfo { spaces: 0, tabs: 0 };
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
        line_info.spaces += 1;
      } else if c == '\t' {
        line_info.tabs += 1;
      } else {
        at_bol = false;
      }
    } else if c == '\n' {
      at_bol = true;
    }
  }

  Ok(line_info)
}

pub fn write_new_file(
  reader: &mut dyn Read,
  writer: &mut dyn Write,
  new_bol: BeginningOfLine,
  tab_size: usize,
  round_down: bool,
) -> Result<LineInfo, Box<dyn Error>> {
  let mut line_info = LineInfo { spaces: 0, tabs: 0 };
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
        s = untabify(&s);

        if new_bol == BeginningOfLine::Tabs {
          let (t, num_spaces) = tabify(&s);

          s = t;
          line_info.tabs += s.len() - num_spaces;
          line_info.spaces += num_spaces;
        } else {
          line_info.spaces += s.len();
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

  Ok(line_info)
}

#[cfg(test)]
#[path = "spacer_tests.rs"]
mod spacer_tests;
