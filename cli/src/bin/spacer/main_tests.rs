use super::*;

#[test]
fn test_run_tabs() {
  let temp_dir = tempfile::tempdir().unwrap();
  let input_path = temp_dir.path().join("input_file.txt");
  let input_file = input_path.to_str().unwrap();

  std::fs::write(input_file, "\t\tabc\r").unwrap();

  run(
    input_file,
    None,
    Some(BeginningOfLineArg::Spaces),
    4,
    4,
    true,
  )
  .unwrap();

  temp_dir.close().unwrap();
}

#[test]
fn test_run_status_only() {
  let temp_dir = tempfile::tempdir().unwrap();
  let input_path = temp_dir.path().join("input_file.txt");
  let input_file = input_path.to_str().unwrap();

  std::fs::write(input_file, "\t\tabc\r").unwrap();

  run(input_file, None, None, 4, 4, false).unwrap();

  temp_dir.close().unwrap();
}

#[test]
fn test_run_mixed() {
  let temp_dir = tempfile::tempdir().unwrap();
  let output_path = temp_dir.path().join("output_file.txt");
  let input_path = temp_dir.path().join("input_file.txt");
  let input_file = input_path.to_str().unwrap();

  std::fs::write(input_file, "\t  abc\r").unwrap();

  run(
    input_file,
    Some(output_path.to_str().unwrap()),
    Some(BeginningOfLineArg::Spaces),
    4,
    4,
    true,
  )
  .unwrap();

  temp_dir.close().unwrap();
}
