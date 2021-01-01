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

        let buffer = match input {
            Input::File(file) => {
                if files_count > 1 {
                    writeln!(writer, "==> {} <==", file)?;
                }

                // Move the contents of the file into a cloneable buffer so the
                // buffer's bytes or lines of the file can be counted without
                // clearing the original buffer.
                let mut buffer: Vec<u8> = Vec::new();
                let mut f = File::open(file)?;
                f.read_to_end(&mut buffer)?;

                buffer
            },
            Input::Stdin => {
                if files_count > 1 {
                    writeln!(writer, "==> standard input <==")?;
                }

                // Move the contents of the standard input into a cloneable
                // buffer so the buffer's bytes or lines of the file can be
                // counted without clearing the original buffer.
                let mut buffer: Vec<u8> = Vec::new();
                let mut stdin = io::stdin();
                stdin.read_to_end(&mut buffer)?;

                buffer
            },
        };

        let reader: BufReader<&[u8]> = BufReader::new(buffer.as_ref());

        // Determine the total number of bytes or lines in the buffer. This is
        // essential in offsetting the beginning of the file.
        let count = match flags {
            Flags::LinesCount(_) => line_count(buffer.clone()),
            Flags::BytesCount(_) => byte_count(buffer.clone()),
        };

        read_stream(flags, reader, &mut writer, count)?;
    }

    Ok(())
}

/// Read from a stream, truncated at a number of lines or bytes from the end,
/// and write back to a stream
fn read_stream<R: Read, W: Write>(
    flags: &Flags, mut reader: BufReader<R>, writer: &mut W, count: usize,
) -> Result<(), io::Error> {
    match flags {
        Flags::LinesCount(lines_count) => {
            let difference = count - lines_count.to_owned();

            for line in reader.lines().skip(difference) {
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
            let difference = count - bytes_count.to_owned();

            let mut buffer = Vec::new();

            reader.fill_buf()?;
            reader.consume(difference);

            reader.read_to_end(&mut buffer)?;

            writer.write_all(&buffer)?;
        },
    }

    Ok(())
}

/// Count the total number of lines in the provided buffer.
fn line_count(buffer: Vec<u8>) -> usize {
    let reader: BufReader<&[u8]> = BufReader::new(buffer.as_ref());

    reader.lines().count()
}

/// Count the total number of bytes in the provided buffer.
fn byte_count(buffer: Vec<u8>) -> usize {
    let reader: BufReader<&[u8]> = BufReader::new(buffer.as_ref());

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
        let count = line_count(buffer.to_vec());

        read_stream(&flags, BufReader::new(&buffer[..]), &mut out, count).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), "bar\nbaz\n".to_string());
    }

    #[test]
    fn read_stream_bytes_count() {
        let buffer = b"foo\nbar\nbaz";
        let flags = Flags::BytesCount(2);
        let mut out = Vec::new();
        let count = byte_count(buffer.to_vec());

        read_stream(&flags, BufReader::new(&buffer[..]), &mut out, count).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), "az".to_string());
    }
}
