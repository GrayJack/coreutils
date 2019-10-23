use std::{
    env::current_dir,
    fs::File,
    io::{prelude::BufRead, stdin, stdout, BufReader, Write},
    process,
};

use clap::{load_yaml, App, ArgMatches};

mod tab_stops;
use tab_stops::*;

struct Unexpand {
    all: bool,
    tabs: TabStops,
}

impl Unexpand {
    fn from_matches(matches: &ArgMatches) -> Self {
        let mut all = matches.is_present("all");
        let first_only = matches.is_present("first_only");
        let tabs_str = matches.value_of("tabs");
        let tabs = TabStops::new(tabs_str);

        if first_only {
            all = false;
        } else if tabs_str.is_some() {
            all = true;
        }

        Unexpand { all, tabs }
    }

    fn unexpand_line(self: &mut Self, line: String) -> String {
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
                        new_line.push_str("\t");
                        spaces = 0;
                    }
                }
                b'\x08' => {
                    spaces -= !!spaces;
                    column -= !!column;
                }
                _ => {
                    column -= spaces;
                    new_line.push_str(String::from(" ").repeat(spaces as usize).as_str());
                    spaces = 0;
                    new_line.push(c as char);
                    convert &= self.all;
                }
            };
        }

        new_line.push_str("\n");

        new_line
    }
}

fn main() {
    let yaml = load_yaml!("unexpand.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let mut unexpand = Unexpand::from_matches(&matches);
    let cwd = match current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("rm: error reading current working directory: {}", err);
            process::exit(1);
        }
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

    for file_path in files {
        if file_path == "-" {
            let stdin = stdin();
            for line in stdin.lock().lines() {
                stdout.write_all(unexpand.unexpand_line(line.unwrap()).as_bytes())
                    .expect("write error");
                stdout.flush().expect("write error");
            }
        } else {
            let fd = File::open(file_path).unwrap();
            let reader = BufReader::new(fd);
            for line in reader.lines() {
                stdout.write_all(unexpand.unexpand_line(line.unwrap()).as_bytes())
                    .expect("write error");
                stdout.flush().expect("write error");
            }
        }
    }
}

#[test]
fn unexpand_lines() {
    let mut instance = Unexpand { all: false, tabs: TabStops::new(Some("2")) };
    assert_eq!(instance.unexpand_line(String::from("    c")), String::from("\t\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("  c")), String::from("\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("  c  c")), String::from("\tc  c\n"));
    assert_eq!(instance.unexpand_line(String::from("   c    c")), String::from("\t c    c\n"));

    let mut instance = Unexpand { all: true, tabs: TabStops::new(Some("2")) };
    assert_eq!(instance.unexpand_line(String::from("    c")), String::from("\t\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("  c")), String::from("\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("  c  c")), String::from("\tc\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("   c    c")), String::from("\t c\t\tc\n"));

    let mut instance = Unexpand { all: true, tabs: TabStops::new(Some("8"))};
    assert_eq!(instance.unexpand_line(String::from("    c")), String::from("    c\n"));
    assert_eq!(instance.unexpand_line(String::from("  c")), String::from("  c\n"));
    assert_eq!(instance.unexpand_line(String::from("  c  c")), String::from("  c  c\n"));
    assert_eq!(instance.unexpand_line(String::from("   c    c")), String::from("   c    c\n"));
    assert_eq!(instance.unexpand_line(String::from("        c")), String::from("\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("        c        c")), String::from("\tc\tc\n"));

    let mut instance = Unexpand { all: true, tabs: TabStops::new(Some("2,+4"))};
    assert_eq!(instance.unexpand_line(String::from("  c")), String::from("\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("          c")), String::from("\t\t\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("  c    c")), String::from("\tc\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("   c    c")), String::from("\t c\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("    c    c")), String::from("\t  c\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("      c    c")), String::from("\t\tc\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("      c        c")), String::from("\t\tc\t\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("      c        c    ")), String::from("\t\tc\t\tc\t\n"));

    let mut instance = Unexpand { all: true, tabs: TabStops::new(Some("2,/4"))};
    assert_eq!(instance.unexpand_line(String::from("  c")), String::from("\tc\n"));
    assert_eq!(instance.unexpand_line(String::from("    c    c")), String::from("\t\tc\tc\n"));

    let mut instance = Unexpand { all: true, tabs: TabStops::new(Some("2 4 6"))};
    assert_eq!(instance.unexpand_line(String::from("      c")), String::from("\t\t\tc\n"));
}
