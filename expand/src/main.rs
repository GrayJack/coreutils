use std::{
    env::current_dir,
    fs::File,
    io::{prelude::BufRead, stdin, stdout, BufReader, Write},
    process,
};

use clap::{load_yaml, App, AppSettings::ColoredHelp, ArgMatches};

mod tab_stops;
use tab_stops::*;

fn main() {
    let yaml = load_yaml!("expand.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();
    let mut expand = Expand::from_matches(&matches);
    let cwd = match current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("expand: error reading current working directory: {}", err);
            process::exit(1);
        },
    };

    let files = match matches.values_of("FILE") {
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
        eprintln!("expand: read error: {}", err);
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
                    .write_all(expand.expand_line(&line.unwrap_or_else(read_error)).as_bytes())
                    .unwrap_or_else(write_error);
                stdout.flush().unwrap_or_else(write_error);
            }
        } else {
            let fd = File::open(file_path).unwrap();
            let reader = BufReader::new(fd);
            for line in reader.lines() {
                stdout
                    .write_all(expand.expand_line(&line.unwrap_or_else(read_error)).as_bytes())
                    .unwrap_or_else(write_error);
                stdout.flush().unwrap_or_else(write_error);
            }
        }
    }
}

struct Expand {
    initial:  bool,
    tabstops: TabStops,
}

impl Expand {
    fn from_matches(matches: &ArgMatches) -> Self {
        let initial = matches.is_present("initial");

        let tabs_str = matches.value_of("tabs");
        let tabstops = match TabStops::new(tabs_str) {
            Ok(tab_stops) => tab_stops,
            Err(err_message) => {
                eprintln!("{}", err_message);
                std::process::exit(1);
            },
        };

        Expand { initial, tabstops }
    }

    fn expand_line(self: &mut Self, line: &str) -> String {
        let mut convert = true;
        let mut column = 0;
        let mut new_line: String = String::new();


        for c in line.bytes() {
            match c {
                b'\t' => {
                    let repeat = self.tabstops.repetable.unwrap();
                    let offset = match self.tabstops.offset {
                        Some(o) => o,
                        None => 0,
                    };
                    let spaces = repeat + offset - column % repeat;
                    column += spaces;
                    if convert {
                        new_line.push_str(String::from(" ").repeat(spaces as usize).as_str());
                    } else {
                        new_line.push('\t');
                    }
                },
                b'\x08' => {
                    column = if column > 0 { column - 1 } else { 0 };
                    new_line.pop();
                },
                _ => {
                    column += 1;
                    convert &= !self.initial;
                    new_line.push(c as char);
                },
            };
        }
        new_line.push_str("\n");
        new_line
    }
}
