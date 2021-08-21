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
        .version("1.0.0-20120712.0")
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
    let line_info = read_eol_info(&mut reader)?;

    print!(
        "'{}', {}, {} lines",
        input_file,
        if line_info.num_endings > 1 {
            "mixed"
        } else if line_info.cr > 0 {
            "cr"
        } else if line_info.lf > 0 {
            "lf"
        } else {
            "crlf"
        },
        line_info.num_lines
    );

    if let Some(eol_arg) = eol_arg {
        let new_eol = match eol_arg {
            EndOfLineArg::Auto => line_info.get_common_eol(),
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
#[path = "main_tests.rs"]
mod tests;
