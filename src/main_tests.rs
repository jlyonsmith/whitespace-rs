mod tests {
  use crate::*;

  #[test]
  fn test_read_eol_info_lf() {
    let line_info = read_eol_info(&mut "\n".as_bytes()).unwrap();

    assert_eq!(
      line_info,
      LineInfo {
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
    let line_info = read_eol_info(&mut "\r".as_bytes()).unwrap();

    assert_eq!(
      line_info,
      LineInfo {
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
    let line_info = read_eol_info(&mut "\r\n".as_bytes()).unwrap();

    assert_eq!(
      line_info,
      LineInfo {
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
    let line_info = read_eol_info(&mut "\n\r\n\r".as_bytes()).unwrap();

    assert_eq!(
      line_info,
      LineInfo {
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
    let num_lines = write_new_file(&mut input, &mut output, EndOfLine::CrLf).unwrap();

    assert_eq!(num_lines, 4);
    assert_eq!(String::from_utf8(output).unwrap(), "abc\r\n\r\n\r\n")
  }

  #[test]
  #[should_panic]
  fn test_write_new_file_bad_arg() {
    let mut input = "".as_bytes();
    let mut output = Vec::new();
    write_new_file(&mut input, &mut output, EndOfLine::Auto).unwrap();
  }

  #[test]
  fn test_run_auto() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("output_file.txt");
    let input_path = temp_dir.path().join("input_file.txt");
    let input_file = input_path.to_str().unwrap();

    std::fs::write(input_file, "abc\nxyz\r\n\r\n123\r\r\r").unwrap();

    run(
      input_file,
      Some(output_path.to_str().unwrap()),
      Some(EndOfLine::Auto),
    )
    .unwrap();

    temp_dir.close().unwrap();
  }

  #[test]
  fn test_run_just_status() {
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("input_file.txt");
    let input_file = input_path.to_str().unwrap();

    std::fs::write(input_file, "abc\r\n").unwrap();

    run(input_file, None, None).unwrap();

    temp_dir.close().unwrap();
  }

  #[test]
  fn test_run_crlf() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("output_file.txt");
    let input_path = temp_dir.path().join("input_file.txt");
    let input_file = input_path.to_str().unwrap();

    std::fs::write(input_file, "abc\r\n").unwrap();

    run(
      input_file,
      Some(output_path.to_str().unwrap()),
      Some(EndOfLine::CrLf),
    )
    .unwrap();

    temp_dir.close().unwrap();
  }

  #[test]
  fn test_run_cr() {
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("input_file.txt");
    let input_file = input_path.to_str().unwrap();

    std::fs::write(input_file, "abc\r").unwrap();

    run(input_file, None, Some(EndOfLine::CrLf)).unwrap();

    temp_dir.close().unwrap();
  }
}
