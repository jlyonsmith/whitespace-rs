use super::*;

#[test]
fn test_read_bol_info() {
  let line_info = read_bol_info(&mut "  \txyz\n".as_bytes()).unwrap();

  assert_eq!(line_info, LineInfo { spaces: 2, tabs: 1 });
}

#[test]
fn test_write_new_file_round_down() {
  let mut input = " a\n  x\n    \n".as_bytes();
  let mut output = Vec::new();
  let line_info = write_new_file(&mut input, &mut output, BeginningOfLine::Tabs, 2, true).unwrap();

  assert_eq!(line_info, LineInfo { spaces: 0, tabs: 3 });
  assert_eq!(String::from_utf8(output).unwrap(), "a\n\tx\n\t\t\n");
}

#[test]
fn test_write_new_file() {
  let mut input = " a\n   x\n    \n".as_bytes();
  let mut output = Vec::new();
  let line_info = write_new_file(&mut input, &mut output, BeginningOfLine::Tabs, 2, false).unwrap();

  assert_eq!(line_info, LineInfo { spaces: 2, tabs: 3 });
  assert_eq!(String::from_utf8(output).unwrap(), " a\n\t x\n\t\t\n");
}

#[test]
fn test_write_new_file_tabs() {
  let mut input = "\ta\n \t x\n\t\t\n".as_bytes();
  let mut output = Vec::new();
  let line_info =
    write_new_file(&mut input, &mut output, BeginningOfLine::Spaces, 2, true).unwrap();

  assert_eq!(line_info, LineInfo { spaces: 9, tabs: 0 });
  assert_eq!(String::from_utf8(output).unwrap(), "  a\n   x\n    \n");
}
