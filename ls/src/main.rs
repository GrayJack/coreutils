use clap::ArgMatches;

use coreutils_core::os::group::Group;
use coreutils_core::os::passwd::Passwd;

use std::string::String;
use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::{fs, path, process};
use std::time::SystemTime;

use ansi_term::Color;

extern crate chrono;

use chrono::prelude::{DateTime, Utc};

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let files = matches.values_of("FILE").unwrap();
    let flags = LsFlags::from_matches(&matches);

    let mut exit_code = 0;

    for file in files {
        match fs::read_dir(file) {
            Ok(dir) => {
                let mut dir: Vec<_> = dir.map(|r| r.unwrap()).collect();

                if flags.time {
                    dir.sort_by_key(sort_by_time);
                } else {
                    // Sort the directory entries by file name by default
                    dir.sort_by_key(sort_by_name);
                }

                if flags.reverse {
                    dir.reverse();
                }

                if flags.list {
                    exit_code = print_list(dir, flags);
                } else {
                    exit_code = print_default(dir, flags);
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
fn print_default(dir: Vec<fs::DirEntry>, flags: LsFlags) -> i32 {
    let exit_code = 1;

    for entry in dir {
        let file_name = get_file_name(&entry);

        if is_hidden(&file_name) && !flags.all {
            continue;
        }

        print!("{} ", file_name);
    }
    println!();

    exit_code
}

/// Prints information about the provided file in a long format
fn print_list(dir: Vec<fs::DirEntry>, flags: LsFlags) -> i32 {
    let mut exit_code = 1;

    for entry in dir {
        match fs::symlink_metadata(entry.path()) {
            Ok(metadata) => {
                let file_name = get_file_name(&entry);

                if is_hidden(&file_name) && !flags.all {
                    continue;
                }

                let mode = metadata.permissions().mode();
                let perms = unix_mode::to_string(mode);

                let modified = metadata.modified().unwrap();
                let modified_datetime: DateTime<Utc> = modified.into();

                let user = Passwd::from_uid(metadata.st_uid()).unwrap();

                let group = Group::from_gid(metadata.st_gid()).unwrap();

                let mut links = 1;

                if metadata.is_dir() {
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
                    metadata.len(),
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

/// Gets a file name from a directory entry and adds appropriate formatting
fn get_file_name(file: &fs::DirEntry) -> String {
    let mut file_name = file.file_name().into_string().unwrap();

    let metadata = fs::symlink_metadata(file.path());

    if let Ok(metadata) = metadata {
        if is_executable(&file.path()) {
            file_name = add_executable_color(file_name);
        }

        if metadata.file_type().is_symlink() {
            file_name = add_symlink_color(file_name);
            let symlink = fs::read_link(file.path());

            if let Ok(symlink) = symlink {
                let mut symlink_name = String::from(symlink.to_str().unwrap());

                if is_executable(&symlink) {
                    symlink_name = add_executable_color(symlink_name);
                }

                file_name = format!("{} -> {}", file_name, symlink_name);
            }
        }

        if metadata.is_dir() {
            file_name = add_directory_color(file_name);
        }
    }

    file_name
}

/// Adds a bold green color to a file name to represent an executable
fn add_executable_color(file_name: String) -> String {
    Color::Green.bold().paint(file_name).to_string()
}

/// Adds a bold blue color to a directory name
fn add_directory_color(directory_name: String) -> String {
    Color::Blue.bold().paint(directory_name).to_string()
}

/// Adds a bold cyan color to a file name to represent a symlink
fn add_symlink_color(symlink_name: String) -> String {
    Color::Cyan.bold().paint(symlink_name).to_string()
}

/// Check if a path is an executable file
fn is_executable(path: &path::PathBuf) -> bool {
    let mut result = false;

    let metadata = fs::symlink_metadata(path);

    if let Ok(metadata) = metadata {
        result = metadata.is_file() && metadata.permissions().mode() & 0o111 != 0;
    }

    result
}

/// Checks if a string looks like a hidden unix file
fn is_hidden(str: &str) -> bool {
    str.starts_with('.')
}

/// Sort a list of directories by file name alphabetically
fn sort_by_name(dir: &fs::DirEntry) -> String {
    let file_name = dir.file_name().into_string().unwrap();

    file_name.to_lowercase()
}

/// Sort a list of directories by modification time
fn sort_by_time(dir: &fs::DirEntry) -> SystemTime {
    let metadata = fs::metadata(dir.path()).expect("Failed to get metadata");

    metadata.modified().expect("Failed to get file's modification time")
}

#[derive(Default, Copy, Clone)]
struct LsFlags {
    all: bool,
    list: bool,
    reverse: bool,
    time: bool,
}

impl LsFlags {
    fn from_matches(matches: &ArgMatches<'_>) -> Self {
        let all = matches.is_present("all");
        let list = matches.is_present("list");
        let reverse = matches.is_present("reverse");
        let time = matches.is_present("time");

        LsFlags { all, list, reverse, time }
    }
}
