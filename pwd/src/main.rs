use std::process;

use coreutils_core::env;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let curr_dir = {
        // The local path we get from environment variable PWD
        match env::current_dir_logical() {
            Ok(dir) => {
                if matches.is_present("logical") {
                    dir
                } else if let Ok(d) = dir.canonicalize() {
                    d
                } else {
                    eprintln!("pwd: Failed to get absolute current directory.");
                    process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("pwd: Failed to get current directory. {}", e);
                process::exit(1);
            },
        }
    };

    println!("{}", curr_dir.display());
}
