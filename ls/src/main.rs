use std::{
    fs,
    io::{self, BufWriter, Write},
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::PathBuf,
    process,
    string::String,
};

use coreutils_core::bstr::{BString, ByteSlice};

extern crate chrono;

use term_grid::Direction;

mod cli;
mod file;
mod flags;
mod output;
mod table;

use file::{File, Files};
use flags::Flags;
use output::{default, grid, list};

fn main() {
    let matches = cli::create_app().get_matches();

    let files = matches.values_of("FILE").unwrap();
    let flags = Flags::from_matches(&matches);

    let mut exit_code = 0;

    let mut writer = BufWriter::new(io::stdout());

    let multiple = files.len() > 1;

    for (i, file) in files.enumerate() {
        if !flags.directory && multiple {
            if i != 0 {
                match writeln!(writer) {
                    Ok(_) => {},
                    Err(err) => {
                        eprintln!("ls: {}", err);
                        process::exit(1);
                    },
                }
            }

            match writeln!(writer, "{}:", file) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("ls: {}", err);
                    process::exit(1);
                },
            }
        }

        let mut result: Files;

        let path = PathBuf::from(file);

        if flags.directory || path.is_file() {
            result = Vec::new();

            let item = if flags.directory {
                File::from_name(BString::from(path.as_os_str().as_bytes()), path, flags)
            } else {
                File::from(PathBuf::from(file), flags)
            };

            match item {
                Ok(item) => {
                    result.push(item);
                },
                Err(err) => {
                    eprintln!("ls: cannot access {}: {}", err, file);
                    process::exit(1);
                },
            }
        } else {
            match fs::read_dir(file) {
                Ok(dir) => {
                    result = dir
                        // Collect information about the file or directory
                        .map(|entry| File::from(entry.unwrap().path(), flags).unwrap())
                        // Hide hidden files and directories if `-a` or `-A` flags
                        // weren't provided
                        .filter(|file| !File::is_hidden(&file.name.as_bstr()) || flags.show_hidden())
                        .collect();

                    if !flags.no_sort {
                        sort(&mut result, &flags);
                    }
                },
                Err(err) => {
                    eprintln!("ls: cannot access '{}': {}", file, err);
                    exit_code = 1;

                    break;
                },
            }

            if !flags.directory && (flags.all || flags.no_sort) {
                // Retrieve the current directories information. This must
                // be canonicalized in case the path is relative.
                let current = PathBuf::from(file).canonicalize();

                let current = match current {
                    Ok(current) => current,
                    Err(err) => {
                        eprintln!("ls: {}", err);
                        process::exit(1);
                    },
                };

                let dot = File::from_name(BString::from("."), current.clone(), flags);

                let dot = match dot {
                    Ok(dot) => dot,
                    Err(err) => {
                        eprintln!("ls: {}", err);
                        process::exit(1);
                    },
                };

                // Retrieve the parent path. Default to the current path if the
                // parent doesn't exist
                let parent_path = match dot.path.parent() {
                    Some(parent) => parent,
                    None => current.as_path(),
                };

                let dot_dot =
                    File::from_name(BString::from(".."), PathBuf::from(parent_path), flags);

                let dot_dot = match dot_dot {
                    Ok(dot_dot) => dot_dot,
                    Err(err) => {
                        eprintln!("ls: {}", err);
                        process::exit(1);
                    },
                };

                result.insert(0, dot);
                result.insert(1, dot_dot);
            }
        }

        if flags.show_list() {
            match list(result, &mut writer, flags) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("ls: cannot access '{}': {}", file, err);
                    exit_code = 1;
                },
            }
        } else if flags.show_grid() {
            let direction = if flags.order_left_to_right && !flags.order_top_to_bottom {
                Direction::LeftToRight
            } else {
                Direction::TopToBottom
            };

            match grid(result, &mut writer, direction) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("ls: cannot access '{}': {}", file, err);
                    exit_code = 1;
                },
            }
        } else {
            match default(result, &mut writer, flags) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("ls: cannot access '{}': {}", file, err);
                    exit_code = 1;
                },
            }
        }
    }

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

/// Sort a list of files based on the provided flags.
fn sort(files: &mut Files, flags: &Flags) {
    if flags.time {
        if flags.last_accessed {
            files.sort_by_key(sort_by_access_time);
        } else if flags.file_status_modification {
            files.sort_by_key(sort_by_last_changed_time)
        } else {
            files.sort_by_key(sort_by_time);
        }
        files.reverse();
    } else if flags.sort_size {
        files.sort_by_key(sort_by_size);
        files.reverse();
    } else {
        // Sort the directory entries by file name by default
        files.sort_by_key(sort_by_name);
    }

    if flags.reverse {
        files.reverse();
    }
}

/// Sort a list of files by last accessed time
fn sort_by_access_time(file: &File) -> i64 { file.metadata.atime() }

/// Sort a list of files by last change of file status information
fn sort_by_last_changed_time(file: &File) -> i64 { file.metadata.ctime() }

/// Sort a list of files by file name alphabetically
fn sort_by_name(file: &File) -> String {
    file.name.to_string().to_lowercase().trim_start_matches('.').to_string()
}

/// Sort a list of files by size
fn sort_by_size(file: &File) -> u64 { file.metadata.len() }

/// Sort a list of directories by modification time
fn sort_by_time(file: &File) -> i64 { file.metadata.mtime() }
