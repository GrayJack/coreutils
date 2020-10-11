use std::{
    fs::File,
    io::{prelude::*, stdin, BufReader, ErrorKind},
};

use clap::ArgMatches;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let flags = SortFlags::from_matches(&matches);

    let filenames: Vec<String> = match matches.values_of("FILE") {
        Some(m) => m.map(String::from).collect(),
        None => unimplemented!("Implement input from stdin"),
    };

    let exit_code = 0;

    let mut lines: Vec<String> = filenames
        .into_iter()
        .filter_map(|filename| File::open(filename).ok())
        .flat_map(|file| BufReader::new(file).lines())
        .filter_map(|l| l.ok())
        .collect();
    lines.sort();
    for (line_number, line) in lines.into_iter().enumerate() {
        print_line(line, flags, line_number);
    }

    std::process::exit(exit_code);
}

#[derive(Debug, Clone, Copy)]
struct SortFlags {
}

impl SortFlags {
    pub fn from_matches(matches: &ArgMatches) -> Self {
        SortFlags {}
    }
}

fn print_line(line: String, flags: SortFlags, line_number: usize) {
    println!("{:6}  {}", line_number, line);
}
