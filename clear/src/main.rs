use std::{
    io::{stdout, Write},
    process,
};

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let x_flag = matches.is_present("x");

    if x_flag {
        match stdout().lock().write(b"\x1b[H\x1b[3J") {
            Ok(_) => (),
            Err(err) => {
                eprintln!("clear: failed to execute: {}", err);
                process::exit(1);
            },
        };
        return;
    }

    match stdout().lock().write(b"\x1b[3J\x1b[H\x1b[2J") {
        Ok(_) => (),
        Err(err) => {
            eprintln!("clear: failed to execute: {}", err);
            process::exit(1);
        },
    };
}
