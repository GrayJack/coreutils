use std::{
    fs, io,
    path::{Path, PathBuf},
    process,
};

use clap::{load_yaml, App, AppSettings::ColoredHelp, ArgMatches};

fn main() {
    let yaml = load_yaml!("rmdir.yml");
    let matches = App::from_yaml(yaml)
        .settings(&[ColoredHelp])
        .help_message("Display help information")
        .version_message("Display version information")
        .get_matches();

    let dirs: Vec<PathBuf> = matches.values_of("DIRECTORY").unwrap().map(PathBuf::from).collect();

    let flags = RmDirFlags::from_matches(&matches);
    let mut ret_val = 0;

    for dir in dirs {
        if let Err(err) = rmdir(&dir, flags) {
            eprintln!("rmdir: failed to remove {}: {}", err.0.display(), err.1);
            ret_val = 1;
        };
    }

    process::exit(ret_val);
}

#[derive(Clone, Copy, Debug)]
struct RmDirFlags {
    verbose: bool,
    parents: bool,
    ignore:  bool,
}

impl RmDirFlags {
    fn from_matches(matches: &ArgMatches<'_>) -> Self {
        RmDirFlags {
            verbose: matches.is_present("verbose"),
            parents: matches.is_present("parents"),
            ignore:  matches.is_present("ignore-fail-nonempty"),
        }
    }
}

#[derive(Debug)]
struct RmdirError(PathBuf, io::Error);

fn rmdir(dir: &PathBuf, flags: RmDirFlags) -> Result<(), RmdirError> {
    let full_dir = match dir.canonicalize() {
        Ok(f) => f,
        Err(err) => return Err(RmdirError(dir.clone(), err)),
    };

    if flags.parents {
        let empty_path = Path::new("");
        let mut path = dir.clone();
        loop {
            let full_path = match dir.canonicalize() {
                Ok(f) => f,
                Err(err) => return Err(RmdirError(path, err)),
            };

            if path == empty_path {
                return Ok(()); // there are no more parents
            }

            // For verbose we display the full path
            if flags.verbose {
                println!("rmdir: removing directory '{}'", full_path.display());
            }

            if flags.ignore {
                match fs::remove_dir_all(&path) {
                    Ok(_) => {
                        if flags.verbose {
                            println!(
                                "rmdir: removed all {} directory content",
                                full_path.display()
                            );
                        }
                    },
                    Err(err) => return Err(RmdirError(full_path, err)),
                }
            } else {
                match fs::remove_dir(&path) {
                    Ok(_) => {
                        if flags.verbose {
                            println!("rmdir: removed directory {}", full_path.display());
                        }
                    },
                    Err(err) => return Err(RmdirError(full_path, err)),
                }
            }

            if !path.pop() {
                return Ok(()); // there are no more parents
            }
        }
    } else if !flags.parents && flags.ignore {
        if flags.verbose {
            println!("rmdir: removing directory '{}'", full_dir.display());
        }

        match fs::remove_dir_all(&dir) {
            Ok(_) => {
                if flags.verbose {
                    println!("rmdir: removed all {} directory content", full_dir.display());
                }
            },
            Err(err) => return Err(RmdirError(full_dir, err)),
        }
    } else {
        if flags.verbose {
            println!("rmdir: removing directory '{}'", full_dir.display());
        }

        match fs::remove_dir(&dir) {
            Ok(_) => {
                if flags.verbose {
                    println!("rmdir: removed directory {}", full_dir.display());
                }
            },
            Err(err) => return Err(RmdirError(full_dir, err)),
        }
    }

    Ok(())
}
