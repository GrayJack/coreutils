use std::io;
use std::process;

use clap::{load_yaml, App, ArgMatches};

const F_IGNORE_FAIL_ON_NONEMPTY: u8 = 0x1;
const F_PARENTS: u8 = 0x2;
const F_VERBOSE: u8 = 0x4;

fn main() {
    let yaml = load_yaml!("rmdir.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let directories: Vec<String> = {
        // Safe to unwrap since we said it is required on clap yaml
        let values = matches.values_of("DIRECTORY").unwrap();
        let mut v = Vec::new();
        for value in values {
            v.push(value.to_owned());
        }
        v
    };

    let flags = parse_flags(&matches);
    let mut ret_val = 0;

    for dir in &directories {
        if let Err(err) = rmdir(dir, flags) {
            eprintln!("rmdir: failed to remove '{}': {}", dir, err);
            ret_val = 1;
        }
    }

    process::exit(ret_val);
}

fn parse_flags(matches: &ArgMatches<'_>) -> u8 {
    let mut flags = 0;

    if matches.is_present("ignore-fail-nonempty") {
        flags |= F_IGNORE_FAIL_ON_NONEMPTY;
    }

    if matches.is_present("parents") {
        flags |= F_PARENTS;
    }

    if matches.is_present("verbose") {
        flags |= F_VERBOSE;
    }

    flags
}

fn rmdir(dirname: &str, flags: u8) -> io::Result<()> {
    use std::fs;
    use std::path::PathBuf;

    if (flags & F_VERBOSE) != 0 {
        eprintln!("rmdir: removing directory, '{}'", dirname);
    }

    let mut path = PathBuf::from(dirname);

    if (flags & F_IGNORE_FAIL_ON_NONEMPTY) != 0 {
        fs::remove_dir_all(&path)?;
    } else {
        fs::remove_dir(&path)?;
    }

    if (flags & F_PARENTS) != 0 {
        loop {
            if !path.pop() {
                break; // there are no more parents
            }

            if let Err(err) = fs::remove_dir(&path) {
                return Err(err);
            }
        }
    }

    Ok(())
}
