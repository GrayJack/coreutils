use std::{fs, os::unix::fs::PermissionsExt, process};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("mkdir.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let directories = matches.values_of("DIRECTORY").unwrap();
    let verbose = matches.is_present("verbose");
    let parents = matches.is_present("parents");
    let has_mode = matches.is_present("mode");

    let mut exit_code = 0;

    let mkdir = {
        if parents { fs::create_dir_all } else { fs::create_dir }
    };

    for dir in directories {
        match mkdir(dir) {
            Ok(_) => {
                if verbose {
                    println!("mkdir: created directory '{}'", dir)
                };
                if has_mode {
                    let mode = matches.value_of("mode").unwrap();
                    match fs::metadata(dir) {
                        Ok(v) => {
                            let mut perms = v.permissions();
                            let umode: u32 = mode.parse().unwrap();
                            perms.set_mode(umode);
                        },
                        Err(err) => {
                            eprintln!("mkdir: {}", err);
                            exit_code = 1;
                        },
                    }
                }
            },
            Err(err) => {
                eprintln!("mkdir: cannot create directory '{}': {}", dir, err);
                exit_code = 1;
            },
        }
    }

    if exit_code != 0 {
        process::exit(exit_code);
    }
}
