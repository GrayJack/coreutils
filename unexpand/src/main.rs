use std::{
    env::current_dir,
    fs::File,
    io::{prelude::BufRead, stdin, stdout, BufReader, Write},
    process,
};

use clap::ArgMatches;

#[cfg(test)]
mod tests;

mod cli;
mod tab_stops;
use tab_stops::*;

fn main() {
    let matches = cli::create_app().get_matches();

    let mut unexpand = Unexpand::from_matches(&matches);
    let cwd = match current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("unexpand: error reading current working directory: {}", err);
            process::exit(1);
        },
    };

    let files: Vec<String> = match matches.values_of("FILE") {
        Some(files) => files
            .map(|file| {
                if file == "-" {
                    return String::from("-");
                }

                file.split_whitespace()
                    .map(|s| cwd.join(s.to_string()).to_str().unwrap().to_string())
                    .collect()
            })
            .collect(),
        None => vec!["-".to_string()],
    };

    let mut stdout = stdout();

    let write_error = |err| {
        eprintln!("unexpand: write error: {}", err);
        process::exit(1);
    };

    let read_error = |err| {
        eprintln!("unexpand: read error: {}", err);
        String::new()
    };

    for file_path in files {
        if file_path == "-" {
            let stdin = stdin();
            for line in stdin.lock().lines() {
                stdout
                    .write_all(unexpand.unexpand_line(&line.unwrap_or_else(read_error)).as_bytes())
                    .unwrap_or_else(write_error);
                stdout.flush().unwrap_or_else(write_error);
            }
        } else {
            let fd = File::open(file_path).unwrap();
            let reader = BufReader::new(fd);
            for line in reader.lines() {
                stdout
                    .write_all(unexpand.unexpand_line(&line.unwrap_or_else(read_error)).as_bytes())
                    .unwrap_or_else(write_error);
                stdout.flush().unwrap_or_else(write_error);
            }
        }
    }
}

#[derive(Debug)]
struct Unexpand {
    all:  bool,
    tabs: TabStops,
}

impl Unexpand {
    fn from_matches(matches: &ArgMatches) -> Self {
        let mut all = matches.is_present("all");
        let first_only = matches.is_present("first_only");
        let tabs_str = matches.value_of("tabs");
        let tabs = match TabStops::new(tabs_str) {
            Ok(tab_stops) => tab_stops,
            Err(err_message) => {
                eprintln!("{}", err_message);
                std::process::exit(1);
            },
        };

        if first_only {
            all = false;
        } else if tabs_str.is_some() {
            all = true;
        }

        Unexpand { all, tabs }
    }

    fn unexpand_line(&mut self, line: &str) -> String {
        let mut convert = true;
        let mut spaces: i32 = 0;
        let mut column: i32 = 0;
        let mut new_line: String = String::new();

        for c in line.bytes() {
            match c {
                b' ' => {
                    spaces += 1;
                    column += 1;

                    if self.tabs.is_tab_stop(column as usize) && convert {
                        new_line.push('\t');
                        spaces = 0;
                    }
                },
                b'\x08' => {
                    spaces -= !!spaces;
                    column -= !!column;
                },
                _ => {
                    column -= spaces;
                    new_line.push_str(String::from(" ").repeat(spaces as usize).as_str());
                    spaces = 0;
                    new_line.push(c as char);
                    convert &= self.all;
                },
            };
        }

        new_line.push('\n');

        new_line
    }
}
