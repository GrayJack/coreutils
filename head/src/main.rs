use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read, Write},
};

use clap::{load_yaml, value_t, App, AppSettings::ColoredHelp, ArgMatches, ErrorKind};

const DEFAULT_LINES_COUNT: usize = 10;
const NEW_LINE: u8 = 0xA;

fn main() {
    let yaml = load_yaml!("head.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let flags = Flags::from_matches(&matches);
    let input = Input::from_matches(&matches);

    head(&flags, input).unwrap_or_else(|e| {
        eprintln!("head: {}", e);
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

/// Input is either a list of one or more files, or STDIN
enum Input {
    Files(Vec<String>),
    Stdin,
}

impl Input {
    /// Parse arguments into an Input enum.
    fn from_matches(matches: &ArgMatches) -> Self {
        if let Some(files) = matches.values_of("FILE") {
            Self::Files(files.map(|f| f.into()).collect())
        } else {
            Self::Stdin
        }
    }
}

/// Return the head of our input, truncated at a number of lines or bytes
fn head(flags: &Flags, input: Input) -> Result<(), io::Error> {
    match input {
        Input::Files(files) => {
            let files_count = files.len();
            for (i, file) in files.iter().enumerate() {
                if files_count > 1 {
                    if i != 0 {
                        println!();
                    }
                    println!("==> {} <==", file);
                }
                let f = File::open(file)?;
                let reader = BufReader::new(f);
                read_stream(flags, reader, &mut io::stdout())?;
            }
        },
        Input::Stdin => {
            let stdin = io::stdin();
            let reader = BufReader::new(stdin.lock());
            read_stream(flags, reader, &mut io::stdout())?;
        },
    }
    Ok(())
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
        let buffer = "foo\nbar\nbaz".as_bytes();
        let flags = Flags::LinesCount(2);
        let mut out = Vec::new();

        read_stream(&flags, BufReader::new(buffer), &mut out).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), "foo\nbar\n".to_string());
    }

    #[test]
    fn read_stream_bytes_count() {
        let buffer = "foo\nbar\nbaz".as_bytes();
        let flags = Flags::BytesCount(2);
        let mut out = Vec::new();

        read_stream(&flags, BufReader::new(buffer), &mut out).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), "fo".to_string());
    }
}
