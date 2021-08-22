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
      num_endings: 1
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
      num_endings: 1
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
      num_endings: 1
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
      num_endings: 3
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
