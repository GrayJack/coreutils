use std::{
    fs::OpenOptions,
    io::{self, BufReader, BufWriter, Read, Write},
    process,
};

use clap::{ArgMatches, Values};

use coreutils_core::libc::{signal, SIGINT, SIG_IGN};

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();
    let flags = Flags::from_matches(&matches);
    let file_arg = matches.values_of("FILE");

    if flags.ignore_interrupts {
        unsafe { signal(SIGINT, SIG_IGN) };
    }

    let exit_code = process_input(file_arg, &flags);

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

/// Processes the input and output based on the provided flags.
fn process_input(file_arg: Option<Values>, flags: &Flags) -> i32 {
    let mut exit_code = 0;

    let mut files: Vec<&str> = Vec::new();

    if flags.append {
        files = match file_arg {
            Some(matches) => matches.collect(),
            None => {
                eprintln!("tee: no files provided");
                exit_code = 1;
                return exit_code;
            },
        };
    }

    let mut input_buffer: Vec<u8> = Vec::new();
    let mut stdin = io::stdin();
    match stdin.read_to_end(&mut input_buffer) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("tee: {}", err);
            exit_code = 1;
            return exit_code;
        },
    }

    if flags.append {
        for path in files {
            let file = match OpenOptions::new().write(true).create(true).append(true).open(path) {
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

            match copy_buffer(reader, &mut writer) {
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

        match copy_buffer(reader, &mut writer) {
            Ok(_) => {},
            Err(err) => {
                eprintln!("tee: {}", err);
                exit_code = 1;
            },
        };
    }

    exit_code
}

/// Writes the contents of input buffer reader to the provided writer.
fn copy_buffer<R: Read, W: Write>(mut reader: BufReader<R>, writer: &mut W) -> io::Result<()> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    writer.write_all(&buffer)?;

    Ok(())
}

struct Flags {
    pub append: bool,
    pub ignore_interrupts: bool,
}

impl Flags {
    pub fn from_matches(matches: &ArgMatches<'_>) -> Self {
        let append = matches.is_present("append");
        let ignore_interrupts = matches.is_present("ignore");

        Flags { append, ignore_interrupts }
    }
}

#[cfg(test)]
mod tests;
