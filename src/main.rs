use clap::{arg_enum, value_t, App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::{Read, Seek, SeekFrom, Write};
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
    #[derive(PartialEq, Debug, Clone, Copy)]
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
                .possible_values(&EndOfLine::variants())
                .case_insensitive(true)
                .required(false),
        )
        .get_matches();

    let input_file = matches.value_of("input_file").unwrap();
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

    if let Ok(mut new_eol) = value_t!(matches, "new_eol", EndOfLine) {
        match new_eol {
            EndOfLine::Auto => {
                let mut n = line_info.lf;

                if line_info.crlf > n {
                    n = line_info.crlf;
                    new_eol = EndOfLine::CrLf;
                }

                if line_info.cr > n {
                    new_eol = EndOfLine::Cr;
                }
            }
            _ => (),
        };

        reader.seek(SeekFrom::Start(0))?;

        let output_file = matches.value_of("output_file");
        let mut writer: Box<dyn Write> = match output_file {
            Some(path) => Box::new(BufWriter::new(File::create(Path::new(path))?)),
            None => Box::new(std::io::stdout()),
        };
        let num_lines = write_new_file(&mut reader, &mut writer, new_eol)?;

        println!(
            " -> '{}', {}, {} lines",
            if let Some(file) = output_file {
                file
            } else {
                "STDOUT"
            },
            new_eol.to_string().to_lowercase(),
            num_lines
        )
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
struct LineInfo {
    cr: usize,
    lf: usize,
    crlf: usize,
    num_lines: usize,
    num_endings: usize,
}

impl Eq for LineInfo {}

fn read_eol_info(reader: &mut dyn Read) -> Result<LineInfo, Box<dyn Error>> {
    let mut line_info = LineInfo {
        cr: 0,
        lf: 0,
        crlf: 0,
        num_endings: 0,
        num_lines: 1,
    };
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

fn write_new_file(
    reader: &mut dyn Read,
    writer: &mut dyn Write,
    new_eol: EndOfLine,
) -> Result<usize, Box<dyn Error>> {
    let mut num_lines = 1;
    let newline_chars = match new_eol {
        EndOfLine::Cr => "\r".as_bytes(),
        EndOfLine::Lf => "\n".as_bytes(),
        EndOfLine::CrLf => "\r\n".as_bytes(),
        _ => panic!(),
    };
    let mut decoder = UnsafeDecoder::new(reader.bytes()).peekable();
    let mut buf = [0u8; 4];

    loop {
        let c;

        match decoder.next() {
            Some(value) => c = value?,
            None => break,
        };
        if c == '\r' {
            if matches!(decoder.peek(), Some(Ok(c)) if *c == '\n') {
                decoder.next();
            }

            num_lines += 1;
            writer.write(newline_chars)?;
        } else if c == '\n' {
            num_lines += 1;
            writer.write(newline_chars)?;
        } else {
            writer.write(c.encode_utf8(&mut buf).as_bytes())?;
        }
    }
    writer.flush()?;

    Ok(num_lines)
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod main_tests;
