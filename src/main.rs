use clap::{arg_enum, App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use utf8_decode::UnsafeDecoder;

fn main() -> Result<(), Box<dyn Error>> {
    let result = run();

    if let Err(ref err) = result {
        eprintln!("error: {}", err);
    }

    result
}

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum EndOfLine {
        Cr,
        Lf,
        CrLf,
        Auto,
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Ender")
        .version("1.0.0-20120712.0")
        .author("John Lyon-Smith")
        .about("End of line normalizer.  Defaults to reporting types of endings.")
        .arg(
            Arg::with_name("input_file")
                .help("Input file in UTF-8 format. Uses STDIN if not specified.")
                .value_name("FILE")
                .index(1)
                .required(false),
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
                .possible_values(&EndOfLine::variants())
                .case_insensitive(true)
                .required(false),
        )
        .get_matches();

    let line_info = read_eol_info(matches.value_of("input_file"))?;

    println!("{:?}", line_info);

    Ok(())
}

#[derive(Debug)]
struct LineInfo {
    cr: usize,
    lf: usize,
    crlf: usize,
    num_lines: usize,
    num_endings: usize,
}

fn read_eol_info(input_path: Option<&str>) -> Result<LineInfo, Box<dyn Error>> {
    let mut line_info = LineInfo {
        cr: 0,
        lf: 0,
        crlf: 0,
        num_endings: 0,
        num_lines: 0,
    };
    let reader: Box<dyn Read> = Box::new(File::open(Path::new(input_path.unwrap()))?);
    let mut decoder = UnsafeDecoder::new(reader.bytes()).peekable();

    loop {
        let c;
        match decoder.next() {
            Some(value) => c = value?,
            None => break,
        };
        if c == '\r' {
            if matches!(decoder.peek(), Some(Ok(c)) if *c == '\n') {
                line_info.crlf += 1;
                decoder.next();
            } else {
                line_info.cr += 1;
            }

            line_info.num_lines += 1;
        } else if c == '\n' {
            line_info.lf += 1;
            line_info.num_lines += 1;
        }
    }

    line_info.num_endings =
        (line_info.cr > 0) as usize + (line_info.lf > 0) as usize + (line_info.crlf > 0) as usize;

    Ok(line_info)
}
