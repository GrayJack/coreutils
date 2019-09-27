use std::path::PathBuf;
use std::io;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

use clap::{load_yaml, App};

struct CatArgs {
    number_nonblank: bool,
    number: bool,
    show_ends: bool,
    squeeze_blank: bool,
    show_tabs: bool,
    show_nonprinting: bool
}

impl Default for CatArgs {
    fn default() -> CatArgs {
        CatArgs {
            number_nonblank: false,
            number: false,
            show_ends: false,
            squeeze_blank: false,
            show_tabs: false,
            show_nonprinting: false
        }
    } 
}

fn main() {
    let yaml = load_yaml!("cat.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let files = {
        if let Some(vals) = matches.values_of("FILE") {
            vals.map(|value| PathBuf::from(value.to_owned())).collect()
        } else {
            vec![PathBuf::from("-")]
        }
    };

    let mut args: CatArgs = CatArgs::default();

    if matches.is_present("show_all") {
        args.show_nonprinting = true;
        args.show_ends = true;
        args.show_tabs = true;
    }

    if matches.is_present("number_nonblank") {
        args.number_nonblank = true;
    }

    if matches.is_present("show_ends_nonprinting") {
        args.show_ends = true;
        args.show_nonprinting = true;
    }

    if matches.is_present("show_ends") {
        args.show_ends = true;
    }

    if matches.is_present("number") {
        args.number = true;
    }

    if matches.is_present("squeeze_blank") {
        args.squeeze_blank = true;
    }

    if matches.is_present("show_tabs_nonprinting") {
        args.show_tabs = true;
        args.show_nonprinting = true;
    }

    if matches.is_present("show_tabs") {
        args.show_tabs = true;
    }

    if matches.is_present("show_nonprinting") {
        args.show_nonprinting = true;
    }

    for file in files {
        match cat(&file, &args) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("cat: Failed to read from {:?}.\n{}", file, e);
            }
        }
    }
}

fn cat(file: &PathBuf, args: &CatArgs) -> io::Result<()> {  
    let file = File::open(file)?;
    let reader = BufReader::new(file);

    let mut lines = reader.lines();

    let mut line_no = 1;

    while let Some(line) = lines.next() {
        let line = line?;

        if args.number && !args.number_nonblank {
            print!("{:6} ", line_no);
            line_no += 1;
        } else if args.number_nonblank && line != "" {
            print!("{:6} ", line_no);
            line_no += 1;
        }

        println!("{}{}", line, if args.show_ends { "$" } else { "" });
    }

    Ok(())
}
