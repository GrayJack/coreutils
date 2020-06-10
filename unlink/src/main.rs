use std::{fs, process};

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let path = matches.value_of("FILE").unwrap();

    // Note: std::fs::remove_file corresponds to unlink(2) at time of this writing, but that
    // may change in the future according to the documentation.
    if let Err(err) = fs::remove_file(path) {
        eprintln!("unlink: cannot unlink '{}': {}", path, err);
        process::exit(1);
    }
}
