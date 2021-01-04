use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    process,
};

use clap::ArgMatches;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();
    let flags = Flags::from_matches(&matches);

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
