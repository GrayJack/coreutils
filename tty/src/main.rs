use std::process;

use coreutils_core::{
    tty::{FileDescriptor, Error::*, TTYName},
};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("tty.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

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
            },
        }
    }

    if !desc_stdin.is_tty() {
        process::exit(1);
    }
}
