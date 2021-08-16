use clap::{arg_enum, value_t, App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use utf8_decode::UnsafeDecoder;

// {grcov-excl-start}
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
            Arg::with_name("new_bol")
                .help("Standardize line beginnings")
                .long("new-bol")
                .short("n")
                .takes_value(true)
                .possible_values(&BeginningOfLine::variants())
                .case_insensitive(true),
        )
        .arg(
            Arg::with_name("tab_size")
                .help("Tab size to assume when converting tabs")
                .long("tab-size")
                .short("t")
                .takes_value(true)
                .value_name("TAB_SIZE")
                .default_value("2"),
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
        value_t!(matches, "new_bol", BeginningOfLine).ok(),
        usize::from_str_radix(matches.value_of("tab_size").unwrap(), 10)?,
        matches.is_present("round_down"),
    );

    if let Err(ref err) = result {
        eprintln!("error: {}", err);
    }

    result
}

arg_enum! {
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub enum BeginningOfLine {
        Tabs,
        Spaces,
        Auto,
    }
}
// {grcov-excl-end}

fn run(
    input_file: &str,
    output_file: Option<&str>,
    new_bol: Option<BeginningOfLine>,
    tab_size: usize,
    round_down: bool,
) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(File::open(Path::new(input_file))?);
    let line_info = read_bol_info(&mut reader)?;
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
        bol_type(line_info.spaces, line_info.tabs),
    );

    if let Some(new_bol) = new_bol {
        reader.seek(SeekFrom::Start(0))?;

        let mut writer: Box<dyn Write> = match output_file {
            Some(path) => Box::new(BufWriter::new(File::create(Path::new(path))?)),
            None => Box::new(std::io::stdout()),
        };
        let line_info = write_new_file(&mut reader, &mut writer, new_bol, tab_size, round_down)?;

        println!(
            " -> '{}', {}",
            if let Some(file) = output_file {
                file
            } else {
                "STDOUT"
            },
            bol_type(line_info.spaces, line_info.tabs)
        )
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
struct LineInfo {
    spaces: usize,
    tabs: usize,
}

impl Eq for LineInfo {}

fn read_bol_info(reader: &mut dyn Read) -> Result<LineInfo, Box<dyn Error>> {
    let mut line_info = LineInfo { spaces: 0, tabs: 0 };
    let mut decoder = UnsafeDecoder::new(reader.bytes()).peekable();
    let mut at_bol = true;

    loop {
        let c;
        match decoder.next() {
            Some(value) => c = value?,
            None => break,
        };

        if at_bol {
            if c == ' ' {
                line_info.spaces += 1;
            } else if c == '\t' {
                line_info.tabs += 1;
            } else {
                at_bol = false;
            }
        } else if c == '\n' {
            at_bol = true;
        }
    }

    Ok(line_info)
}

fn write_new_file(
    reader: &mut dyn Read,
    writer: &mut dyn Write,
    new_bol: BeginningOfLine,
    tab_size: usize,
    round_down: bool,
) -> Result<LineInfo, Box<dyn Error>> {
    let mut line_info = LineInfo { spaces: 0, tabs: 0 };
    let mut decoder = UnsafeDecoder::new(reader.bytes()).peekable();
    let mut buf = [0u8; 4];
    let mut s = String::new();
    let mut at_bol = true;
    let untabify = |s: &str| -> String {
        let mut t = String::new();

        for c in s.chars() {
            if c == '\t' {
                t.push_str(&" ".repeat(tab_size - (t.len() % tab_size)));
            } else {
                t.push(c);
            }
        }

        t
    };
    let tabify = |s: &str| -> (_, _) {
        let mut num_spaces = 0;
        let mut t = String::new();

        for c in s.chars() {
            if c == ' ' {
                num_spaces += 1;
            }

            if num_spaces % tab_size == 0 {
                t.push('\t');
                num_spaces = 0
            }
        }

        if num_spaces > 0 {
            if !round_down {
                t.push_str(&" ".repeat(num_spaces));
            } else {
                num_spaces = 0;
            }
        }

        (t, num_spaces)
    };

    loop {
        let c;

        match decoder.next() {
            Some(value) => c = value?,
            None => break,
        };
        if at_bol {
            if c == ' ' || c == '\t' {
                s.push(c);
            } else {
                s = untabify(&s);

                if new_bol == BeginningOfLine::Tabs {
                    let (t, num_spaces) = tabify(&s);

                    s = t;
                    line_info.tabs += s.len() - num_spaces;
                    line_info.spaces += num_spaces;
                } else {
                    line_info.spaces += s.len();
                }

                writer.write(s.as_bytes())?;
                writer.write(c.encode_utf8(&mut buf).as_bytes())?;

                if c == '\n' {
                    s.clear();
                } else {
                    at_bol = false;
                }
            }
        } else {
            writer.write(c.encode_utf8(&mut buf).as_bytes())?;

            if c == '\n' {
                s.clear();
                at_bol = true;
            }
        }
    }
    writer.flush()?;

    Ok(line_info)
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod main_tests;
