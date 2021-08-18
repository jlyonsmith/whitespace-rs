use super::*;

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
    Some(EndOfLine::Lf),
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

#[test]
fn test_run_lf() {
  let temp_dir = tempfile::tempdir().unwrap();
  let input_path = temp_dir.path().join("input_file.txt");
  let input_file = input_path.to_str().unwrap();

  std::fs::write(input_file, "abc\n").unwrap();

  run(input_file, None, Some(EndOfLine::CrLf)).unwrap();

  temp_dir.close().unwrap();
}
