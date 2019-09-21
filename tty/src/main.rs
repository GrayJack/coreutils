use std::process;

use coreutils_core::{tty::{isatty, TTYName, Error::*}, file_descriptor::FileDescriptor};

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
            Err(ref err) if err == &NotTTY => println!("{}", err),
            _ => process::exit(1)
        }
    }

    if isatty(desc_stdin) {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
