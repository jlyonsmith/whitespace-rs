use clap::{Arg, ArgAction, Command, ValueEnum};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use whitespace_rs::spacer::*;

// {grcov-excl-start}
#[derive(PartialEq, Debug, Clone, ValueEnum)]
/// Types of line beginnings
pub enum BeginningOfLineArg {
    Tabs,
    Spaces,
    Auto,
}

fn main() {
    let matches = Command::new("Spacer")
        .version("2.1.2+20210904.0")
        .author("John Lyon-Smith")
        .about(
            "Beginning of line normalizer. Defaults to reporting BOL type for the file; tabs, spaces, or mixed.",
        )
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
                .value_name("FILE"),
        )
        .arg(
            Arg::new("bol_arg")
                .help("Standardize line beginnings")
                .long("new-bol")
                .short('n')
                .value_parser(clap::builder::EnumValueParser::<BeginningOfLineArg>::new())
        )
        .arg(
            Arg::new("tab_size")
                .help("Tab size for both input and output file")
                .long("tab-size")
                .short('t')
                .value_name("TAB_SIZE")
                .default_value("4"),
        )
        .arg(
            Arg::new("round_down")
                .help("When tabifying, rounds extra spaces down to a whole number of tabs")
                .long("round-down")
                .short('r')
                .action(ArgAction::SetTrue)
        )
        .get_matches();

    let result = run(
        matches.get_one::<String>("input_file").unwrap(),
        matches
            .get_one::<String>("output_file")
            .map(|s_ref| s_ref.as_str()),
        matches.get_one::<BeginningOfLineArg>("bol_arg"),
        usize::from_str_radix(matches.get_one::<String>("tab_size").unwrap(), 10).unwrap_or(4),
        matches.get_flag("round_down"),
    );

    if let Err(ref err) = result {
        eprintln!("error: {}", err);
        std::process::exit(-1);
    }
}
// {grcov-excl-end}

pub fn run(
    input_file: &str,
    output_file: Option<&str>,
    bol_arg: Option<&BeginningOfLineArg>,
    tab_size: usize,
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
    let mut new_bol_info: Option<BolInfo> = None;

    if let Some(bol_arg) = bol_arg {
        let new_bol = match bol_arg {
            BeginningOfLineArg::Auto => bol_info.get_common_bol(tab_size, round_down),
            BeginningOfLineArg::Tabs => BeginningOfLine::Tabs(tab_size, round_down),
            BeginningOfLineArg::Spaces => BeginningOfLine::Spaces(tab_size),
        };

        reader.seek(SeekFrom::Start(0))?;

        let mut writer: Box<dyn Write> = match output_file {
            Some(path) => Box::new(BufWriter::new(File::create(Path::new(path))?)),
            None => Box::new(std::io::stdout()),
        };
        new_bol_info = Some(write_new_bols(&mut reader, &mut writer, new_bol)?);
        println!();
    }

    eprint!(
        "'{}', {}",
        input_file,
        bol_type(bol_info.spaces, bol_info.tabs),
    );

    if let Some(new_bol_info) = new_bol_info {
        eprintln!(
            " -> '{}', {}",
            if let Some(file) = output_file {
                file
            } else {
                "STDOUT"
            },
            bol_type(new_bol_info.spaces, new_bol_info.tabs)
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
    fn test_run_tabs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input_path = temp_dir.path().join("input_file.txt");
        let input_file = input_path.to_str().unwrap();

        std::fs::write(input_file, "\t\tabc\r").unwrap();

        run(input_file, None, Some(&BeginningOfLineArg::Spaces), 4, true).unwrap();

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_run_status_only() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input_path = temp_dir.path().join("input_file.txt");
        let input_file = input_path.to_str().unwrap();

        std::fs::write(input_file, "\t\tabc\r").unwrap();

        run(input_file, None, None, 4, false).unwrap();

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_run_auto_spaces() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("output_file.txt");
        let input_path = temp_dir.path().join("input_file.txt");
        let input_file = input_path.to_str().unwrap();

        std::fs::write(input_file, "\t  abc\r").unwrap();

        run(
            input_file,
            Some(output_path.to_str().unwrap()),
            Some(&BeginningOfLineArg::Auto),
            2,
            true,
        )
        .unwrap();

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_run_auto_tabs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("output_file.txt");
        let input_path = temp_dir.path().join("input_file.txt");
        let input_file = input_path.to_str().unwrap();

        std::fs::write(input_file, "\t\n\t\n\t\t abc\r").unwrap();

        run(
            input_file,
            Some(output_path.to_str().unwrap()),
            Some(&BeginningOfLineArg::Auto),
            2,
            true,
        )
        .unwrap();

        temp_dir.close().unwrap();
    }
}
