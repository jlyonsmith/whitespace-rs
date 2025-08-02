use clap::{Arg, Command, ValueEnum};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use whitespace_rs::ender::*;

// {grcov-excl-start}
/// Types of line endings
#[derive(PartialEq, Debug, Clone, Copy, ValueEnum)]
pub enum EndOfLineArg {
    Cr,
    Lf,
    CrLf,
    Auto,
}

impl std::fmt::Display for EndOfLineArg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EndOfLineArg::Cr => write!(f, "cr"),
            EndOfLineArg::Lf => write!(f, "lf"),
            EndOfLineArg::CrLf => write!(f, "cr-lf"),
            EndOfLineArg::Auto => write!(f, "auto"),
        }
    }
}

fn main() {
    let matches = Command::new("Ender")
        .version("2.1.2+20210904.0")
        .author("John Lyon-Smith")
        .about("End of line normalizer.  Defaults to reporting types of endings.")
        .arg(
            Arg::new("input_file")
                .help("Input file in UTF-8 format.")
                .value_name("FILE")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::new("output_file")
                .help("Output file in UTF-8 format.  Uses STDOUT if not specified")
                .long("output")
                .short('o')
                .value_name("FILE")
                .required(false),
        )
        .arg(
            Arg::new("new_eol")
                .help("Write new line endings.")
                .long("new-eol")
                .short('n')
                .value_parser(clap::builder::EnumValueParser::<EndOfLineArg>::new())
                .required(false),
        )
        .get_matches();

    let result = run(
        matches.get_one::<String>("input_file").unwrap(),
        matches
            .get_one::<String>("output_file")
            .map(|s_ref| s_ref.as_str()),
        matches.get_one::<EndOfLineArg>("new_eol"),
    );

    if let Err(ref err) = result {
        eprintln!("error: {}", err);
        std::process::exit(-1);
    }
}
// {grcov-excl-end}

fn run(
    input_file: &str,
    output_file: Option<&str>,
    eol_arg: Option<&EndOfLineArg>,
) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(File::open(Path::new(input_file))?);
    let eol_info = read_eol_info(&mut reader)?;
    let mut num_lines = eol_info.num_lines;

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

        num_lines = write_new_eols(&mut reader, &mut writer, new_eol)?;
        println!();
    }

    eprint!(
        "'{}', {}, {} lines",
        input_file,
        if eol_info.num_endings() > 1 {
            "mixed"
        } else if eol_info.cr > 0 {
            "cr"
        } else if eol_info.lf > 0 {
            "lf"
        } else {
            "cr-lf"
        },
        eol_info.num_lines
    );

    if let Some(eol_arg) = eol_arg {
        eprintln!(
            " -> '{}', {}, {} lines",
            if let Some(file) = output_file {
                file
            } else {
                "STDOUT"
            },
            eol_arg.to_string().to_lowercase(),
            num_lines
        )
    } else {
        eprintln!();
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
            Some(&EndOfLineArg::Auto),
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
            Some(&EndOfLineArg::Lf),
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

        run(input_file, None, Some(&EndOfLineArg::CrLf)).unwrap();

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_run_lf() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input_path = temp_dir.path().join("input_file.txt");
        let input_file = input_path.to_str().unwrap();

        std::fs::write(input_file, "abc\n").unwrap();

        run(input_file, None, Some(&EndOfLineArg::CrLf)).unwrap();

        temp_dir.close().unwrap();
    }
}
