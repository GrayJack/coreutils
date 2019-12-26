use std::{
    fs::File,
    io::{prelude::*, stdin, BufReader, ErrorKind},
};

use clap::{load_yaml, App, AppSettings::ColoredHelp, ArgMatches};

fn main() {
    let yaml = load_yaml!("cat.yml");
    let matches = App::from_yaml(yaml)
        .settings(&[ColoredHelp])
        .help_message("Display help information")
        .version_message("Display version information")
        .get_matches();

    let flags = CatFlags::from_matches(&matches);

    let files: Vec<String> = match matches.values_of("FILE") {
        Some(m) => m.map(String::from).collect(),
        None => vec![String::from("-")],
    };

    let mut exit_code = 0;

    let mut line_number = 1;
    let mut last_line_empty = false;
    for filename in files.iter() {
        if filename == "-" {
            loop {
                let mut line = String::new();

                match stdin().read_line(&mut line) {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        line.pop();
                        print_line(line, flags, &mut line_number, &mut last_line_empty)
                    },
                    Err(error) => eprintln!("cat: stdin: {}", error),
                }
            }
        } else {
            match File::open(filename) {
                Ok(file) => {
                    for line in BufReader::new(file).lines() {
                        let line = match line {
                            Ok(line) => line,
                            Err(error) => {
                                eprintln!("cat: {}: I/O error: {}", filename, error);
                                exit_code = 1;
                                break;
                            },
                        };
                        print_line(line, flags, &mut line_number, &mut last_line_empty);
                    }
                },
                Err(e) => {
                    exit_code = 1;
                    match e.kind() {
                        ErrorKind::NotFound => {
                            eprintln!("cat: {}: No such file or directory", filename)
                        },
                        ErrorKind::PermissionDenied => {
                            eprintln!("cat: {}: Permission denied", filename)
                        },
                        _ => eprintln!("cat: {}: Unknown error", filename),
                    }
                },
            };
        }
    }

    std::process::exit(exit_code);
}

#[derive(Debug, Clone, Copy)]
struct CatFlags {
    pub number: bool,
    pub number_nonblank: bool,
    pub show_ends: bool,
    pub squeeze_blank: bool,
}

impl CatFlags {
    pub fn from_matches(matches: &ArgMatches) -> Self {
        CatFlags {
            number: matches.is_present("number"),
            number_nonblank: matches.is_present("numberNonblank"),
            show_ends: matches.is_present("showEnds"),
            squeeze_blank: matches.is_present("squeezeBlank"),
        }
    }
}

fn print_line(line: String, flags: CatFlags, line_number: &mut usize, last_line_empty: &mut bool) {
    if flags.squeeze_blank {
        if line.is_empty() {
            if !*last_line_empty {
                if !flags.number_nonblank && flags.number {
                    if flags.show_ends {
                        println!("{:6}  $", line_number);
                    } else {
                        println!("{:6}  ", line_number);
                    }

                    *line_number += 1;
                } else if flags.show_ends {
                    println!("$");
                } else {
                    println!();
                }
            }
            *last_line_empty = true;
        } else {
            if flags.number || flags.number_nonblank {
                if flags.show_ends {
                    println!("{:6}  {}$", line_number, line);
                } else {
                    println!("{:6}  {}", line_number, line);
                }
                *line_number += 1;
            } else if flags.show_ends {
                println!("{}$", line);
            } else {
                println!("{}", line);
            }
            *last_line_empty = false;
        }
    } else if flags.number_nonblank {
        if !line.is_empty() {
            if flags.show_ends {
                println!("{:6}  {}$", line_number, line);
            } else {
                println!("{:6}  {}", line_number, line);
            }
            *line_number += 1;
        } else if flags.show_ends {
            println!("$");
        } else {
            println!();
        }
    } else if flags.number {
        if flags.show_ends {
            println!("{:6}  {}$", line_number, line);
        } else {
            println!("{:6}  {}", line_number, line);
        }
        *line_number += 1;
    } else if flags.show_ends {
        println!("{}$", line);
    } else {
        println!("{}", line);
    }
}
