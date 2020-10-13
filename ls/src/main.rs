use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::{fs, process};

use coreutils_core::os::group::Group;
use coreutils_core::os::passwd::Passwd;

extern crate chrono;

use chrono::prelude::{DateTime, Utc};

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let files = matches.values_of("FILE").unwrap();
    let all = matches.is_present("all");
    let list = matches.is_present("list");
    let reverse = matches.is_present("reverse");
    let time = matches.is_present("time");

    let mut exit_code = 0;

    for file in files {
        match fs::read_dir(file) {
            Ok(dir) => {
                let mut dir: Vec<_> = dir.map(|r| r.unwrap()).collect();

                if time {
                    dir.sort_by_key(|dir| {
                        let metadata = fs::metadata(dir.path()).expect("Failed to get metadata");

                        metadata.modified().expect("Failed to get file's modification time")
                    });
                } else {
                    // Sort the directory entries by file name by default
                    dir.sort_by_key(|dir| dir.path());
                }

                if reverse {
                    dir.reverse();
                }

                if list {
                    exit_code = print_list(dir, all);
                } else {
                    exit_code = print_default(dir, all);
                }
            }
            Err(err) => {
                eprintln!("ls: cannot access '{}': {}", file, err);
                exit_code = 1;
            }
        }
    }

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

/// Prints information about a file in the default format
fn print_default(dir: Vec<fs::DirEntry>, all: bool) -> i32 {
    let exit_code = 1;

    for entry in dir {
        let file_name = entry.file_name().into_string().unwrap();

        if is_hidden(&file_name) && !all {
            continue;
        }

        print!("{} ", file_name);
    }
    println!();

    exit_code
}

/// Prints information about the provided file in a long format
fn print_list(dir: Vec<fs::DirEntry>, all: bool) -> i32 {
    let mut exit_code = 1;

    for entry in dir {
        match fs::metadata(entry.path()) {
            Ok(meta_data) => {
                let file_name = entry.file_name().into_string().unwrap();

                if is_hidden(&file_name) && !all {
                    continue;
                }

                let mode = meta_data.permissions().mode();
                let perms = unix_mode::to_string(mode);

                let modified = meta_data.modified().unwrap();
                let modified_datetime: DateTime<Utc> = modified.into();

                let user = Passwd::from_uid(meta_data.st_uid()).unwrap();

                let group = Group::from_gid(meta_data.st_gid()).unwrap();

                let mut links = 1;

                if meta_data.is_dir() {
                    let subdir = fs::read_dir(entry.path());

                    if let Ok(subdir) = subdir {
                        let subdir_map = subdir.map(|r| r.unwrap());

                        links = 2 + subdir_map
                            .filter(|r| fs::metadata(r.path()).unwrap().is_dir())
                            .count();
                    }
                }

                println!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    perms,
                    links,
                    user.name(),
                    group.name(),
                    meta_data.len(),
                    modified_datetime.format("%b %e %k:%M"),
                    file_name,
                );
            }
            Err(err) => {
                eprintln!("ls: {}", err);
                exit_code = 1;
            }
        }
    }

    exit_code
}

/// Checks if a string looks like a hidden unix file
fn is_hidden(str: &str) -> bool {
    str.starts_with('.')
}
