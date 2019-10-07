use std::{
    env::current_dir,
    fs, io,
    path::{Path, PathBuf},
    process,
};

use clap::{load_yaml, App, ArgMatches};

fn main() {
    let yaml = load_yaml!("rmdir.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let pwd = match current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("rmdir: error reading current working directory: {}", err);
            process::exit(1);
        },
    };

    let dirs: Vec<PathBuf> = matches.values_of("DIRECTORY").unwrap().map(PathBuf::from).collect();

    let flags = RmDirFlags::from_matches(&matches);
    let mut ret_val = 0;

    for dir in dirs {
        if rmdir(&dir, &pwd, flags).is_err(){
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

fn rmdir(dir: &PathBuf, pwd: &PathBuf, flags: RmDirFlags) -> io::Result<()> {
    if flags.parents {
        let empty_path = Path::new("");
        let mut path = dir.clone();
        loop {
            if path == empty_path {
                return Ok(()); // there are no more parents
            }

            // For verbose we display the full path
            if flags.verbose {
                println!("rmdir: removing directory '{}'", pwd.join(&path).display());
            }

            if flags.ignore {
                match fs::remove_dir_all(&path) {
                    Ok(_) => {
                        if flags.verbose {
                            println!(
                                "rmdir: removed all {} directory content",
                                pwd.join(&path).display()
                            );
                        }
                    },
                    Err(err) => {
                        eprintln!("rmdir: failed to remove '{}': {}", pwd.join(&path).display(), err);
                        return Err(err);
                    },
                }
            } else {
                match fs::remove_dir(&path) {
                    Ok(_) => {
                        if flags.verbose {
                            println!("rmdir: removed directory {}", pwd.join(&path).display());
                        }
                    },
                    Err(err) => {
                        eprintln!("rmdir: failed to remove '{}': {}", pwd.join(&path).display(), err);
                        return Err(err)
                    },
                }
            }

            if !path.pop() {
                return Ok(()); // there are no more parents
            }
        }
    } else if !flags.parents && flags.ignore {
        if flags.verbose {
            println!("rmdir: removing directory '{}'", pwd.join(dir).display());
        }

        match fs::remove_dir_all(&dir) {
            Ok(_) => {
                if flags.verbose {
                    println!("rmdir: removed all {} directory content", pwd.join(dir).display());
                }
            },
            Err(err) => {
                eprintln!("rmdir: failed to remove '{}': {}", pwd.join(&dir).display(), err);
                return Err(err)
            },
        }
    } else {
        if flags.verbose {
            println!("rmdir: removing directory '{}'", pwd.join(dir).display());
        }

        match fs::remove_dir(&dir) {
            Ok(_) => {
                if flags.verbose {
                    println!("rmdir: removed directory {}", pwd.join(dir).display());
                }
            },
            Err(err) => {
                eprintln!("rmdir: failed to remove '{}': {}", pwd.join(&dir).display(), err);
                return Err(err)
            },
        }
    }

    Ok(())
}
