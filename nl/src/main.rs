use std::{
    env::current_dir,
    io::{prelude::BufRead, stdin, stdout, Write, BufReader},
    fs::File,
    process,
};

use regex::Regex;
use clap::{load_yaml, App, ArgMatches};

#[cfg(test)]
mod tests;

#[derive(Debug)]
enum Style {
    All,
    Nonempty,
    None,
    Regex(Regex),
}

impl PartialEq for Style {
    fn eq(&self, other: &Style) -> bool {
        match (self, other) {
            (&Style::All, &Style::All) |
            (&Style::Nonempty, &Style::Nonempty) |
            (&Style::None, &Style::None) => true,
            (&Style::Regex(ref reg1), &Style::Regex(ref reg2)) => reg1.to_string() == reg2.to_string(),
            (_, _) => false,
        }
    }
}

impl Style {
    fn from_value(value: Option<&str>, default_value: Style) -> Self {
        match value {
            Some("a") => Style::All,
            Some("t") => Style::Nonempty,
            Some("n") => Style::None,
            Some(reg) => if reg.starts_with("p") {
                let regex = Regex::new(&reg[1..]).unwrap_or_else(|err| {
                    eprintln!("{}", err.to_string());
                    std::process::exit(1);
                });

                Style::Regex(regex)
            } else {
                eprintln!("nl: invalid body numbering style: ‘{}’", reg[1..].to_string());
                std::process::exit(1);
            },
            None => default_value,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Format {
    Ln,
    Rn,
    Rz,
}

impl Format {
    fn from_value(value: Option<&str>) -> Self {
        match value {
            Some("ln") => Format::Ln,
            Some("rn") | None => Format::Rn,
            Some("rz") => Format::Rz,
            Some(s) => {
                eprintln!("nl: invalid line numbering format: ‘{}’", s);
                std::process::exit(1);
            }
        }
    }
}

struct NlArgs {
    body_numbering: Style,
    section_delimiter: String,
    footer_numbering: Style,
    header_numbering: Style,
    line_increment: usize,
    join_blank_lines: usize,
    number_format: Format,
    no_renumber: bool,
    number_separator: String,
    starting_line_number: i64,
    number_width: usize,
}

impl NlArgs {
    fn from_matches(matches: &ArgMatches) -> Self {
        let line_increment_str = matches.value_of("line_increment").unwrap_or("1");
        let line_increment = line_increment_str.parse::<usize>().unwrap_or_else(|_err| {
            eprintln!("nl: invalid line number increment: ‘{}’", line_increment_str);
            std::process::exit(1);
        });

        let join_blank_lines_str = matches.value_of("join_blank_lines").unwrap_or("1");
        let join_blank_lines = join_blank_lines_str.parse::<usize>().unwrap_or_else(|_err| {
            eprintln!("nl: invalid line number of blank lines: ‘{}’", join_blank_lines_str);
            std::process::exit(1);
        });

        let starting_line_number_str = matches.value_of("starting_line_number_str").unwrap_or("1");
        let starting_line_number = starting_line_number_str.parse::<i64>().unwrap_or_else(|_err| {
            eprintln!("nl: invalid starting line number: ‘{}’", starting_line_number_str);
            std::process::exit(1);
        });

        let number_width_str = matches.value_of("number_width").unwrap_or("6");
        let number_width = number_width_str.parse::<usize>().unwrap_or_else(|_err| {
            eprintln!("nl: invalid line number field width: ‘{}’", number_width_str);
            std::process::exit(1);
        });

        NlArgs {
            body_numbering: Style::from_value(matches.value_of("body_numbering"), Style::Nonempty),
            section_delimiter: matches.value_of("section_delimiter").unwrap_or("\\:").to_string(),
            footer_numbering: Style::from_value(matches.value_of("footer_numbering"), Style::None),
            header_numbering: Style::from_value(matches.value_of("header_numbering"), Style::None),
            line_increment,
            join_blank_lines,
            number_format: Format::from_value(matches.value_of("number_format")),
            no_renumber: matches.is_present("no_renumber"),
            number_separator: matches.value_of("number_separator").unwrap_or("\t").to_string(),
            starting_line_number,
            number_width,
        }
    }
}

enum Section {
    Body,
    Header,
    Footer,
}

struct SectionDelimiters {
    body: String,
    header: String,
    footer: String,
}

impl SectionDelimiters {
    fn new(delimiter: String) -> SectionDelimiters {
        SectionDelimiters {
            header: delimiter.repeat(3),
            body: delimiter.repeat(2),
            footer: delimiter.repeat(1),
        }
    }
}

struct Nl {
    ind: i64,
    section: Section,
    section_delimiters: SectionDelimiters,
    args: NlArgs
}

impl Nl {
    fn new(args: NlArgs) -> Self {
        let section_delimiters = SectionDelimiters::new(args.section_delimiter.clone());

        Nl {
            ind: args.starting_line_number,
            section: Section::Body,
            section_delimiters,
            args,
        }
    }

    fn convert(&mut self, files: Vec<String>) {
        let mut stdout = stdout();

        let write_error = |err| {
            eprintln!("nl: write error: {}", err);
            process::exit(1);
        };

        let read_error = |err| {
            eprintln!("nl: read error: {}", err);
            String::new()
        };

        for file_path in files {
            if file_path == "-" {
                let stdin = stdin();
                for line in stdin.lock().lines() {
                    stdout
                        .write_fmt(format_args!("{}\n", self.convert_line(line.unwrap_or_else(read_error))))
                        .unwrap_or_else(write_error);
                    stdout.flush().unwrap_or_else(write_error);
                }
            } else {
                let fd = File::open(file_path).unwrap();
                let reader = BufReader::new(fd);
                for line in reader.lines() {
                    stdout
                        .write_fmt(format_args!("{}\n", self.convert_line(line.unwrap_or_else(read_error))))
                        .unwrap_or_else(write_error);
                    stdout.flush().unwrap_or_else(write_error);
                }
            }
        }
    }

    fn convert_line(&mut self, line: String) -> String {
        let is_section_changed = self.check_and_change_section(&line);

        if is_section_changed {
            return String::new();
        }

        let numbering = match self.section {
            Section::Header => &self.args.header_numbering,
            Section::Body => &self.args.body_numbering,
            Section::Footer => &self.args.footer_numbering,
        };

        let should_number: bool = match numbering {
            Style::All => true,
            Style::None => false,
            Style::Nonempty => line != "",
            Style::Regex(re) => re.is_match(line.as_str()),
        };

        let mut new_line = String::new();
        let num_str = self.ind.to_string();
        let whitespaces = if num_str.len() < self.args.number_width {
            self.args.number_width - num_str.len()
        } else {
            0
        };

        if should_number {
            if self.args.number_format == Format::Rn {
                new_line.push_str(&String::from(" ").repeat(whitespaces));
            } else if self.args.number_format == Format::Rz {
                new_line.push_str(&String::from("0").repeat(whitespaces));
            }

            new_line.push_str(&num_str);

            if self.args.number_format == Format::Ln {
                new_line.push_str(&String::from(" ").repeat(whitespaces));
            }

            new_line.push_str(&self.args.number_separator);

            self.ind += self.args.line_increment as i64;
        } else if line != "" {
            println!("{}", self.args.number_width);
            new_line.push_str(&String::from(" ").repeat(self.args.number_width + 1));
        }


        new_line.push_str(&line);

        new_line
    }

    fn check_and_change_section(&mut self, line: &String) -> bool {
        if line == &self.section_delimiters.header {
            self.section = Section::Header;
            self.ind = self.args.starting_line_number;
            return true;
        } else if line == &self.section_delimiters.body {
            self.section = Section::Body;
            self.ind = self.args.starting_line_number;
            return true;
        } else if line == &self.section_delimiters.footer {
            self.section = Section::Footer;
            self.ind = self.args.starting_line_number;
            return true;
        }

        false
    }
}

fn main() {
    let yaml = load_yaml!("nl.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let nl_args = NlArgs::from_matches(&matches);
    let mut nl = Nl::new(nl_args);
    let cwd = match current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("nl: error reading current working directory: {}", err);
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

    nl.convert(files);
}