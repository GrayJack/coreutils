use std::{env, path::PathBuf, process};

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("pwd.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let curr_dir = {
        // The local path we get from environment variable PWD
        match env::var("PWD") {
            Ok(dir) => {
                if matches.is_present("logical") {
                    PathBuf::from(dir)
                } else {
                    // If if physical, canonicalize path
                    let dir = PathBuf::from(dir);
                    match dir.canonicalize() {
                        Ok(d) => d,
                        _ => {
                            eprintln!("Failed to get absolute current directory.");
                            process::exit(1);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get current directory. {:#?}", e);
                process::exit(1);
            }
        }
    };

    let curr_dir = match curr_dir.to_str() {
        Some(s) => s,
        None => {
            eprintln!("Failed to transform to str.");
            process::exit(1);
        }
    };

    println!("{}", curr_dir);
}
