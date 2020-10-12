use std::{fs, process};
use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

use coreutils_core::os::group::Group;
use coreutils_core::os::passwd::Passwd;

extern crate chrono;

use chrono::prelude::{DateTime, Utc};

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let files = matches.values_of("FILE").unwrap();
    let list = matches.is_present("list");
    let all = matches.is_present("all");

    let mut exit_code = 0;

    for file in files {
        match fs::read_dir(file) {
            Ok(dir) => {
                let mut dir: Vec<_> = dir.map(|r| r.unwrap()).collect();

                dir.sort_by_key(|dir| dir.path());

                if list {
                    exit_code = print_list(dir, all);
                } else {
                    exit_code = print_default(dir, all);
                }
            }
            Err(err) => {
                eprintln!("ls: cannot access '{}': {}", file, err);
                exit_code = 1;
            },
        }
    }

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

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

                println!("{}\t1\t{}\t{}\t{}\t{}\t{}",
                    perms,
                    user.name(),
                    group.name(),
                    meta_data.len(),
                    modified_datetime.format("%b %e %k:%M"),
                    file_name,
                );
            },
            Err(err) => {
                eprintln!("ls: {}", err);
                exit_code = 1;
            },
        }
    }

    exit_code
}

/// Checks if a string looks like a hidden unix file
fn is_hidden(str: &str) -> bool {
    str.starts_with('.')
}
