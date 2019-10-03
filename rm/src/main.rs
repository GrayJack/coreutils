use clap::{load_yaml, App, ArgMatches};
use std::env::current_dir;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(Debug, Clone)]
struct Flags {
    pub force: bool,
    pub interactive: bool,
    pub interactive_batch: bool,
    pub preserve_root: bool,
    pub recursive: bool,
    pub dirs: bool,
    pub verbose: bool,
}

impl Flags {
    pub fn from_matches(matches: &ArgMatches) -> Flags {
        let mut flags = Flags {
            force: matches.is_present("force"),
            interactive: matches.is_present("interactive"),
            interactive_batch: matches.is_present("interactiveBatch"),
            preserve_root: !matches.is_present("noPreserveRoot"),
            recursive: matches.is_present("recursive"),
            dirs: matches.is_present("directories"),
            verbose: matches.is_present("verbose"),
        };

        if flags.force {
            flags.interactive = false;
            flags.interactive_batch = false;
        }

        flags
    }
}

fn main() {
    let yaml = load_yaml!("rm.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let cwd = match current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(-1);
        }
    };

    let files: Vec<PathBuf> = {
        // Safe to unwrap since we said it is required on clap yaml
        let values = matches.values_of("FILE").unwrap();
        let mut v = Vec::new();
        for value in values {
            v.push(cwd.join(value.to_owned()));
        }
        v
    };

    let flags = Flags::from_matches(&matches);

    if flags.preserve_root && files.contains(&PathBuf::from("/")) {
        println!(
            "It is dangerous to operate on '/'; use --no-preserve-root to override this failsafe."
        );
        exit(-1);
    }

    if flags.interactive_batch && (files.len() > 3 || flags.recursive) {
        print!("Are you sure you want to do this deletion? [Y/n]: ");
        io::stdout().lock().flush().expect("Could not flush stdout");
        if !input_affirmative() {
            exit(-1);
        }
    }

    match rm(files, &flags) {
        Ok(()) => {}
        Err(msg) => {
            eprintln!("Error: {}", msg);
            exit(-1);
        }
    };
}

fn rm(files: Vec<PathBuf>, flags: &Flags) -> Result<(), String> {
    for file in files {
        if file.is_file() {
            if ask_if_interactive(&file, flags) {
                match fs::remove_file(&file) {
                    Ok(()) => {
                        if flags.verbose {
                            println!("removed {}", file.display());
                        }
                    }
                    Err(err) => {
                        if !flags.force {
                            eprintln!("cannot remove {}, {}", file.display(), err);
                        }
                    }
                };
            }
        } else if file.is_dir() {
            if flags.recursive {
                match remove_dir_all(&file, flags) {
                    Ok(()) => {}
                    Err(err) => {
                        if !flags.force {
                            return Err(err.to_string());
                        }
                    }
                };
            } else if flags.dirs {
                if ask_if_interactive(&file, flags) {
                    match fs::remove_dir(&file) {
                        Ok(()) => {
                            if flags.verbose {
                                println!("removed {}", file.display());
                            }
                        }
                        Err(_) => {}
                    };
                }
            } else {
                if !flags.force {
                    eprintln!("cannot delete {}, it is a directory", file.display());
                }
            }
        } else {
            if !flags.force {
                eprintln!(
                    "cannot remove '{}', no such file or directory",
                    file.display()
                );
            }
        }
    }

    Ok(())
}

fn remove_dir_all(path: &Path, flags: &Flags) -> io::Result<()> {
    let filetype = fs::symlink_metadata(path)?.file_type();
    if filetype.is_symlink() {
        if ask_if_interactive(path, flags) {
            match fs::remove_file(path) {
                Ok(()) => {
                    if flags.verbose {
                        println!("removed {}", path.display());
                    }
                }
                Err(err) => {
                    if !flags.force {
                        eprintln!("cannot remove {}, {}", path.display(), err);
                    }
                }
            };

            Ok(())
        } else {
            Ok(())
        }
    } else {
        remove_dir_all_recursive(path, flags)
    }
}

fn remove_dir_all_recursive(path: &Path, flags: &Flags) -> io::Result<()> {
    for child in fs::read_dir(path)? {
        let child = child?;
        if child.file_type()?.is_dir() {
            remove_dir_all_recursive(&child.path(), flags)?;
        } else {
            if ask_if_interactive(path, flags) {
                match fs::remove_file(&child.path()) {
                    Ok(()) => {
                        if flags.verbose {
                            println!("removed {}", child.path().display());
                        }
                    }
                    Err(err) => {
                        if !flags.force {
                            eprintln!("cannot remove {}, {}", path.display(), err);
                        }
                    }
                };
            }
        }
    }

    if ask_if_interactive(path, flags) {
        match fs::remove_dir(path) {
            Ok(()) => {
                if flags.verbose {
                    println!("removed {}", path.display());
                }
            }
            Err(err) => {
                if !flags.force {
                    eprintln!("cannot remove {}, {}", path.display(), err);
                }
            }
        };
    }

    Ok(())
}

fn ask_if_interactive(file: &Path, flags: &Flags) -> bool {
    if flags.interactive {
        print!(
            "Are you sure you want to delete {}? [Y/n]: ",
            file.display()
        );
        io::stdout().lock().flush().expect("Could not flush stdout");

        input_affirmative()
    } else {
        true
    }
}

fn input_affirmative() -> bool {
    let input = match get_input() {
        Ok(res) => res,
        Err(msg) => {
            eprintln!("Can not read input: {}", msg);
            exit(-1);
        }
    };

    let input = input.trim().to_uppercase();

    input == "Y" || input == "YES" || input == "1"
}

fn get_input() -> Result<String, String> {
    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(_) => {}
        Err(msg) => return Err(msg.to_string()),
    };

    Ok(line)
}
