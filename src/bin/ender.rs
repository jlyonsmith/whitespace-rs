use clap::{arg_enum, value_t, App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use whitespace_rs::ender::*;

// {grcov-excl-start}
arg_enum! {
  #[derive(PartialEq, Debug, Clone, Copy)]
  /// Types of line endings
  pub enum EndOfLineArg {
      Cr,
      Lf,
      CrLf,
      Auto,
  }
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Ender")
        .version("2.0.0+20210823.0")
        .author("John Lyon-Smith")
        .about("End of line normalizer.  Defaults to reporting types of endings.")
        .arg(
            Arg::with_name("input_file")
                .help("Input file in UTF-8 format.")
                .value_name("FILE")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("output_file")
                .help("Output file in UTF-8 format.  Uses STDOUT if not specified")
                .long("output")
                .short("o")
                .takes_value(true)
                .value_name("FILE")
                .required(false),
        )
        .arg(
            Arg::with_name("new_eol")
                .help("Write new line endings.")
                .long("new-eol")
                .short("n")
                .takes_value(true)
                .possible_values(&EndOfLineArg::variants())
                .case_insensitive(true)
                .required(false),
        )
        .get_matches();

    let result = run(
        matches.value_of("input_file").unwrap(),
        matches.value_of("output_file"),
        value_t!(matches, "new_eol", EndOfLineArg).ok(),
    );

    if let Err(ref err) = result {
        eprintln!("error: {}", err);
    }

    result
}
// {grcov-excl-end}

fn run(
    input_file: &str,
    output_file: Option<&str>,
    eol_arg: Option<EndOfLineArg>,
) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(File::open(Path::new(input_file))?);
    let eol_info = read_eol_info(&mut reader)?;

    print!(
        "'{}', {}, {} lines",
        input_file,
        if eol_info.num_endings > 1 {
            "mixed"
        } else if eol_info.cr > 0 {
            "cr"
        } else if eol_info.lf > 0 {
            "lf"
        } else {
            "crlf"
        },
        eol_info.num_lines
    );

    if let Some(eol_arg) = eol_arg {
        let new_eol = match eol_arg {
            EndOfLineArg::Auto => eol_info.get_common_eol(),
            EndOfLineArg::Lf => EndOfLine::Lf,
            EndOfLineArg::Cr => EndOfLine::Cr,
            EndOfLineArg::CrLf => EndOfLine::CrLf,
        };

        reader.seek(SeekFrom::Start(0))?;

        let mut writer: Box<dyn Write> = match output_file {
            Some(path) => Box::new(BufWriter::new(File::create(Path::new(path))?)),
            None => Box::new(std::io::stdout()),
        };
        let num_lines = write_new_eols(&mut reader, &mut writer, new_eol)?;

        println!(
            " -> '{}', {}, {} lines",
            if let Some(file) = output_file {
                file
            } else {
                "STDOUT"
            },
            eol_arg.to_string().to_lowercase(),
            num_lines
        )
    }

    Ok(())
}

#[cfg(test)]
mod tests {
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
            Some(EndOfLineArg::Auto),
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
            Some(EndOfLineArg::Lf),
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

        run(input_file, None, Some(EndOfLineArg::CrLf)).unwrap();

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_run_lf() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input_path = temp_dir.path().join("input_file.txt");
        let input_file = input_path.to_str().unwrap();

        std::fs::write(input_file, "abc\n").unwrap();

        run(input_file, None, Some(EndOfLineArg::CrLf)).unwrap();

        temp_dir.close().unwrap();
    }
}
