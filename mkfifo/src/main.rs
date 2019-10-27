use clap::{load_yaml, App, AppSettings::ColoredHelp};
use coreutils_core::mkfifo::mkfifo;
use std::process;

fn main() {
    let yaml = load_yaml!("mkfifo.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();
    let filepath = matches.value_of("NAME").expect("No filename provided.");
    let mode = matches.value_of("mode").unwrap();
    let mode: u32 = u32::from_str_radix(mode, 8).unwrap_or_else(|_| {
        eprintln!("Mode '{}' is not an octal number.", mode);
        process::exit(1);
    });
    match mkfifo(filepath, mode.into()) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed creating the fifo.\n{}", e);
            process::exit(1);
        },
    }
}
