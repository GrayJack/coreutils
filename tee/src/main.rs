use std::{
    fs::OpenOptions,
    io::{self, BufReader, BufWriter, Read, Write},
    process,
};

use clap::ArgMatches;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();
    let flags = Flags::from_matches(&matches);
    let mut exit_code = 0;

    let mut files: Vec<&str> = Vec::new();

    if flags.append {
        files = match matches.values_of("FILE") {
            Some(matches) => matches.collect(),
            None => {
                eprintln!("tee: no files provided");
                process::exit(1);
            },
        };
    }

    let mut input_buffer: Vec<u8> = Vec::new();
    let mut stdin = io::stdin();
    match stdin.read_to_end(&mut input_buffer) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("tee: {}", err);
            process::exit(1);
        },
    }

    if flags.append {
        for path in files {
            let file = match OpenOptions::new().read(true).write(true).create(true).open(path) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("tee: {}", err);
                    exit_code = 1;
                    break;
                },
            };

            let input = input_buffer.clone();
            let reader: BufReader<&[u8]> = BufReader::new(input.as_ref());
            let mut writer = BufWriter::new(file);

            match tee(reader, &mut writer) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("tee: {}", err);
                    exit_code = 1;
                    break;
                },
            };
        }
    } else {
        let reader: BufReader<&[u8]> = BufReader::new(input_buffer.as_ref());

        let mut writer = BufWriter::new(io::stdout());

        match tee(reader, &mut writer) {
            Ok(_) => {},
            Err(err) => {
                eprintln!("tee: {}", err);
                exit_code = 1;
            },
        };
    }

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

fn tee<R: Read, W: Write>(mut reader: BufReader<R>, writer: &mut W) -> io::Result<()> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    writer.write_all(&buffer)?;

    Ok(())
}

struct Flags {
    pub append: bool,
    pub ignore: bool,
}

impl Flags {
    pub fn from_matches(matches: &ArgMatches<'_>) -> Self {
        let append = matches.is_present("append");
        let ignore = matches.is_present("ignore");

        Flags { append, ignore }
    }
}
