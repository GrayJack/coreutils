use std::process;

use coreutils_core::{
    file_descriptor::FileDescriptor,
    tty::{isatty, Error::*, TTYName},
};

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("tty.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let desc_stdin = FileDescriptor::StdIn;

    let silent_flag = matches.is_present("silent");

    let res = TTYName::new(desc_stdin);

    if !silent_flag {
        match res {
            Ok(tty) => println!("{}", tty),
            Err(err @ NotTTY) => eprintln!("tty: {}", err),
            Err(err) => {
                eprintln!("tty: {}", err);
                process::exit(1)
            }
        }
    }

    if isatty(desc_stdin) {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
