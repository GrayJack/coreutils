extern crate chrono;

use std::{
    io::{self, Write},
    process
};

use chrono::{DateTime, Local};

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("date.yml");
    let _matches = App::from_yaml(yaml).get_matches();

    match date() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("date: Failed to write to stdout.\n{}", e);
            process::exit(1);
        }
    };
}

fn date() -> io::Result<()> {
    let local: DateTime<Local> = Local::now();

    let stdout = io::stdout();
    let mut output = stdout.lock();

    write!(output, "{}", local.format("%a %b %e %k:%M:%S %Z %Y"))?;

    Ok(())
}