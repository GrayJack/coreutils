use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read, Write},
};

use clap::{value_t, ArgMatches, ErrorKind};

mod cli;

const DEFAULT_LINES_COUNT: usize = 10;
const NEW_LINE: u8 = 0xA;

fn main() {
    let matches = cli::create_app().get_matches();

    let flags = Flags::from_matches(&matches);
    let input_list = Input::from_matches(&matches);

    head(&flags, input_list).unwrap_or_else(|_e| {
        std::process::exit(1);
    });
}

/// We truncate the input at either some number of lines or bytes
enum Flags {
    LinesCount(usize),
    BytesCount(usize),
}

impl Flags {
    /// Parse arguments into a Flags enum
    ///
    /// This will exit the program early on invalid args
    fn from_matches(matches: &ArgMatches) -> Self {
        match value_t!(matches, "bytes", usize) {
            Ok(bytes_count) => Flags::BytesCount(bytes_count),
            Err(e) => match e.kind {
                ErrorKind::ValueValidation => e.exit(),
                _ => match value_t!(matches, "lines", usize) {
                    Ok(l) => Flags::LinesCount(l),
                    Err(e) => match e.kind {
                        ErrorKind::ValueValidation => e.exit(),
                        _ => Flags::LinesCount(DEFAULT_LINES_COUNT),
                    },
                },
            },
        }
    }
}

/// Input is either a file, or STDIN
enum Input {
    File(String),
    Stdin,
}

impl Input {
    /// Parse arguments into an Vec of Input enums.
    fn from_matches(matches: &ArgMatches) -> Vec<Self> {
        if let Some(files) = matches.values_of("FILE") {
            files
                .map(|f| if f == "-" { Self::Stdin } else { Self::File(String::from(f)) })
                .collect()
        } else {
            vec![Self::Stdin]
        }
    }
}

/// Return the head of our input, truncated at a number of lines or bytes
fn head(flags: &Flags, input_list: Vec<Input>) -> Result<(), io::Error> {
    let mut err_return = Ok(());
    let files_count = input_list.len();

    for (i, input) in input_list.iter().enumerate() {
        if i > 0 {
            println!();
        }
        match input {
            Input::File(file) => {
                let f = match File::open(file) {
                    Ok(f) => f,
                    Err(err) => {
                        eprintln!("head: Cannot open '{}' for reading: {}", file, err);
                        err_return = Err(err);
                        continue;
                    },
                };

                if files_count > 1 {
                    println!("==> {} <==", file);
                }
                let reader = BufReader::new(f);
                read_stream(flags, reader, &mut io::stdout())?;
            },

            Input::Stdin => {
                if files_count > 1 {
                    println!("==> standard input <==");
                }
                let stdin = io::stdin();
                let reader = BufReader::new(stdin.lock());
                read_stream(flags, reader, &mut io::stdout())?;
            },
        }
    }

    err_return
}

/// Read from a stream, truncated at a number of lines or bytes and write back to a stream
fn read_stream<R: Read, W: Write>(
    flags: &Flags, mut reader: BufReader<R>, writer: &mut W,
) -> Result<(), io::Error> {
    match flags {
        Flags::LinesCount(lines_count) => {
            for _ in 0..*lines_count {
                let mut buffer = Vec::new();
                let bytes_read = reader.read_until(NEW_LINE, &mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                writer.write_all(&buffer)?;
            }
        },
        Flags::BytesCount(bytes_count) => {
            let mut buffer = Vec::new();
            reader.take(*bytes_count as u64).read_to_end(&mut buffer)?;
            writer.write_all(&buffer)?;
        },
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_stream_lines_count() {
        let buffer = b"foo\nbar\nbaz";
        let flags = Flags::LinesCount(2);
        let mut out = Vec::new();

        read_stream(&flags, BufReader::new(&buffer[..]), &mut out).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), "foo\nbar\n".to_string());
    }

    #[test]
    fn read_stream_bytes_count() {
        let buffer = b"foo\nbar\nbaz";
        let flags = Flags::BytesCount(2);
        let mut out = Vec::new();

        read_stream(&flags, BufReader::new(&buffer[..]), &mut out).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), "fo".to_string());
    }
}
