use clap::{load_yaml, value_t, App, ArgMatches, ErrorKind};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};

const DEFAULT_LINES_COUNT: usize = 10;
const NEW_LINE: u8 = 0xA;

fn main() {
    let yaml = load_yaml!("head.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let flags = Flags::parse(&matches);
    let input = Input::parse(&matches);

    head(flags, input).unwrap_or_else(|e| {
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
    fn parse(matches: &ArgMatches) -> Self {
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
    fn parse(matches: &ArgMatches) -> Self {
        if let Some(files) = matches.values_of("FILE") {
            Self::Files(files.map(|f| f.into()).collect())
        } else {
            Self::Stdin
        }
    }
}

/// Return the head of our input, truncated at a number of lines or bytes
fn head(flags: Flags, input: Input) -> Result<(), io::Error> {
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
                read_stream(&flags, reader)?;
            }
        }
        Input::Stdin => {
            let stdin = io::stdin();
            let reader = BufReader::new(stdin.lock());
            read_stream(&flags, reader)?;
        }
    }
    Ok(())
}

/// Read from a stream, truncated at a number of lines or bytes
fn read_stream<R: Read>(flags: &Flags, mut reader: BufReader<R>) -> Result<(), io::Error> {
    match flags {
        Flags::LinesCount(lines_count) => {
            for _ in 0..*lines_count {
                let mut buffer = Vec::new();
                let bytes_read = reader.read_until(NEW_LINE, &mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                io::stdout().write_all(&buffer)?;
            }
        }
        Flags::BytesCount(bytes_count) => {
            let mut buffer = Vec::new();
            reader.take(*bytes_count as u64).read_to_end(&mut buffer)?;
            io::stdout().write_all(&buffer)?;
        }
    }
    Ok(())
}
