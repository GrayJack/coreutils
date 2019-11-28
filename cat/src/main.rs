use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
    path::PathBuf,
};

use clap::{load_yaml, App, ArgMatches};

#[derive(Copy, Clone)]
struct CatArgs {
    number_nonblank: bool, // done
    number: bool,          // done
    show_ends: bool,       // done
    squeeze_blank: bool,   // done
    show_tabs: bool,       // done
}

impl Default for CatArgs {
    fn default() -> CatArgs {
        CatArgs {
            number_nonblank: false,
            number: false,
            show_ends: false,
            squeeze_blank: false,
            show_tabs: false,
        }
    }
}

impl CatArgs {
    pub fn from(matches: &ArgMatches) -> CatArgs {
        CatArgs {
            number_nonblank: matches.is_present("number_nonblank"),
            number: matches.is_present("number"),
            show_ends: matches.is_present("show_all") || matches.is_present("show_ends"),
            squeeze_blank: matches.is_present("squeeze_blank"),
            show_tabs: matches.is_present("show_all") || matches.is_present("show_tabs"),
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

    let args = CatArgs::from(&matches);

    for file in &files {
        let result = {
            if *file == PathBuf::from("-") {
                let stdin = io::stdin();
                cat(stdin.lock(), args)
            } else {
                match File::open(file) {
                    Ok(file) => cat(file, args),
                    Err(e) => Err(e),
                }
            }
        };

        match result {
            Ok(_) => {},
            Err(e) => eprintln!("cat: Failed to read from {:?}.\n{}", file, e),
        }
    }
}

fn cat<R: Read>(file: R, args: CatArgs) -> io::Result<()> {
    let reader = BufReader::new(file);

    let mut lines = reader.lines();

    let mut line_no = 1;

    while let Some(line) = lines.next() {
        let mut line = line?;

        if args.squeeze_blank && line == "" {
            continue;
        }

        if args.number && !args.number_nonblank {
            print!("{:6} ", line_no);
            line_no += 1;
        } else if args.number_nonblank && line != "" {
            print!("{:6} ", line_no);
            line_no += 1;
        }

        if args.show_tabs {
            line = line.replace("\t", "^I");
        }

        println!("{}{}", line, if args.show_ends { "$" } else { "" });
    }

    Ok(())
}
