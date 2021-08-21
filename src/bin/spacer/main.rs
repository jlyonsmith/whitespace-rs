use clap::{arg_enum, value_t, App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use whitespace_rs::spacer::*;

// {grcov-excl-start}
arg_enum! {
  #[derive(PartialEq, Debug, Clone, Copy)]
  /// Types of line beginnings
  pub enum BeginningOfLineArg {
      Tabs,
      Spaces,
      Auto,
  }
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Ender")
        .version("1.0.0-20120712.0")
        .author("John Lyon-Smith")
        .about("End of line normalizer. Defaults to reporting types of endings.")
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
                .value_name("FILE"),
        )
        .arg(
            Arg::with_name("bol_arg")
                .help("Standardize line beginnings")
                .long("new-bol")
                .short("n")
                .takes_value(true)
                .possible_values(&BeginningOfLineArg::variants())
                .case_insensitive(true),
        )
        .arg(
            Arg::with_name("old_tab_size")
                .help("Tab size to assume in the input file")
                .long("old-tab-size")
                .short("ot")
                .takes_value(true)
                .value_name("TAB_SIZE")
                .default_value("4"),
        )
        .arg(
            Arg::with_name("new_tab_size")
                .help("Tab size to assume in the output file")
                .long("new-tab-size")
                .short("nt")
                .takes_value(true)
                .value_name("TAB_SIZE")
                .default_value("4"),
        )
        .arg(
            Arg::with_name("round_down")
                .help("When tabifying, rounds extra spaces down to a whole number of tabs")
                .long("round-down")
                .short("r"),
        )
        .get_matches();

    let result = run(
        matches.value_of("input_file").unwrap(),
        matches.value_of("output_file"),
        value_t!(matches, "bol_arg", BeginningOfLineArg).ok(),
        usize::from_str_radix(matches.value_of("old_tab_size").unwrap(), 10)?,
        usize::from_str_radix(matches.value_of("new_tab_size").unwrap(), 10)?,
        matches.is_present("round_down"),
    );

    if let Err(ref err) = result {
        eprintln!("error: {}", err);
    }

    result
}
// {grcov-excl-end}

pub fn run(
    input_file: &str,
    output_file: Option<&str>,
    bol_arg: Option<BeginningOfLineArg>,
    old_tab_size: usize,
    new_tab_size: usize,
    round_down: bool,
) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(File::open(Path::new(input_file))?);
    let bol_info = read_bol_info(&mut reader)?;
    let bol_type = |s: usize, t: usize| {
        if t > 0 {
            if s > 0 {
                "mixed"
            } else {
                "tabs"
            }
        } else {
            "spaces"
        }
    };

    print!(
        "'{}', {}",
        input_file,
        bol_type(bol_info.spaces, bol_info.tabs),
    );

    if let Some(bol_arg) = bol_arg {
        let new_bol = match bol_arg {
            BeginningOfLineArg::Auto => bol_info.get_common_bol(),
            BeginningOfLineArg::Tabs => BeginningOfLine::Tabs,
            BeginningOfLineArg::Spaces => BeginningOfLine::Spaces,
        };

        reader.seek(SeekFrom::Start(0))?;

        let mut writer: Box<dyn Write> = match output_file {
            Some(path) => Box::new(BufWriter::new(File::create(Path::new(path))?)),
            None => Box::new(std::io::stdout()),
        };
        let bol_info = write_new_bols(
            &mut reader,
            &mut writer,
            new_bol,
            old_tab_size,
            new_tab_size,
            round_down,
        )?;

        println!(
            " -> '{}', {}",
            if let Some(file) = output_file {
                file
            } else {
                "STDOUT"
            },
            bol_type(bol_info.spaces, bol_info.tabs)
        )
    }

    Ok(())
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
