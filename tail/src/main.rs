use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
};

use clap::{value_t, ArgMatches, ErrorKind};

mod cli;

const DEFAULT_LINES_COUNT: usize = 10;

fn main() {
    let matches = cli::create_app().get_matches();

    let flags = Flags::from_matches(&matches);
    let input_list = Input::from_matches(&matches);

    tail(&flags, input_list).unwrap_or_else(|err| {
        eprintln!("tail: {}", err);
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

/// Return the tail of our input, truncated at a number of lines or bytes
fn tail(flags: &Flags, input_list: Vec<Input>) -> io::Result<()> {
    let files_count = input_list.len();
    let mut writer = BufWriter::new(io::stdout());

    for (i, input) in input_list.iter().enumerate() {
        if i > 0 {
            writeln!(writer)?;
        }
        match input {
            Input::File(file) => {
                let f = File::open(file)?;

                if files_count > 1 {
                    writeln!(writer, "==> {} <==", file)?;
                }

                let count = match flags {
                    Flags::LinesCount(_) => line_count(file)?,
                    Flags::BytesCount(_) => byte_count(file)?,
                };

                let reader = BufReader::new(f);
                read_stream(flags, reader, &mut writer, count)?;
            },

            Input::Stdin => {
                if files_count > 1 {
                    writeln!(writer, "==> standard input <==")?;
                }

                let mut buffer = String::new();
                let mut stdin = io::stdin();
                stdin.read_to_string(&mut buffer)?;

                let reader = BufReader::new(buffer.as_bytes());

                let count = match flags {
                    Flags::LinesCount(_) => lines_count_stdin(buffer.clone()),
                    Flags::BytesCount(_) => byte_count_stdin(buffer.clone()),
                };

                read_stream(flags, reader, &mut writer, count)?;
            },
        }
    }

    Ok(())
}

/// Read from a stream, truncated at a number of lines or bytes and write back to a stream
fn read_stream<R: Read, W: Write>(
    flags: &Flags, mut reader: BufReader<R>, writer: &mut W, count: usize,
) -> Result<(), io::Error> {
    match flags {
        Flags::LinesCount(lines_count) => {
            let lines_count_difference = count - lines_count.to_owned();

            for line in reader.lines().skip(lines_count_difference) {
                let line = match line {
                    Ok(line) => line,
                    Err(err) => {
                        return Err(io::Error::new(io::ErrorKind::Other, err));
                    },
                };

                writeln!(writer, "{}", line)?;
            }
        },
        Flags::BytesCount(bytes_count) => {
            let bytes_count_difference = count - bytes_count.to_owned();

            let mut buffer = Vec::new();

            reader.fill_buf()?;
            reader.consume(bytes_count_difference);

            reader.take(*bytes_count as u64).read_to_end(&mut buffer)?;

            writer.write_all(&buffer)?;
        },
    }
    Ok(())
}

fn line_count(file_path: &str) -> io::Result<usize> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    Ok(reader.lines().count())
}

fn lines_count_stdin(buffer: String) -> usize {
    let reader = BufReader::new(buffer.as_bytes());

    reader.lines().count()
}

fn byte_count(file_path: &str) -> io::Result<usize> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    Ok(reader.bytes().count())
}

fn byte_count_stdin(buffer: String) -> usize {
    let reader = BufReader::new(buffer.as_bytes());

    reader.bytes().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_stream_lines_count() {
        let buffer = b"foo\nbar\nbaz";
        let flags = Flags::LinesCount(2);
        let mut out = Vec::new();

        read_stream(&flags, BufReader::new(&buffer[..]), &mut out, 3).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), "bar\nbaz\n".to_string());
    }

    #[test]
    fn read_stream_bytes_count() {
        let buffer = b"foo\nbar\nbaz";
        let flags = Flags::BytesCount(2);
        let mut out = Vec::new();

        read_stream(&flags, BufReader::new(&buffer[..]), &mut out, 11).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), "az".to_string());
    }
}
