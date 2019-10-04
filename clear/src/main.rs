use std::{
    io::{stdout, Write},
    process,
};

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("clear.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // let t = matches.is_present("-T $TERM");
    let x = matches.is_present("x");

    if let (/*false,*/ false) = (/*t,*/ x) {
        let _ = stdout().lock().write(b"\x1b[3J\x1b[H\x1b[2J");
        process::exit(1);
    }

    // if t {
    //     let _ = stdout().lock().write(b"\x1b[H\x1b[2J");
    //     process::exit(1);
    // } else
    if x {
        let _ = stdout().lock().write(b"\x1b[H\x1b[2J");
        process::exit(1);
    } else {
        println!(
            "Usage: clear [options]

Options:
-T TERM     use this instead of $TERM
-V          print curses-version
-x          do not try to clear scrollback"
        );
        process::exit(1);
    }
}
