use std::process;

use coreutils_core::mkfifo::mkfifo;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    // Ok to unwrap because it is required.
    let filepath = matches.value_of("NAME").unwrap();

    // Ok to unwrap because always have a default.
    let mode = matches.value_of("mode").unwrap();
    let mode: u32 = u32::from_str_radix(mode, 8).unwrap_or_else(|_| {
        eprintln!("mkfifo: Invalid mode. '{}' is not an octal number.", mode);
        process::exit(1);
    });

    match mkfifo(filepath, mode) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("mkfifo: Failed creating the fifo.\n{}", e);
            process::exit(1);
        },
    }
}
