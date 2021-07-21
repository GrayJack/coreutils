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

mod cli;
mod file;
mod flags;
mod output;
mod table;

use file::{File, Files};
use flags::Flags;
use output::output;

fn main() {
    let matches = cli::create_app().get_matches();

    let files = matches.values_of("FILE").unwrap();
    let flags = Flags::from_matches(&matches);

    let mut exit_code = 0;

    let mut writer = BufWriter::new(io::stdout());

    if flags.directory {
        let mut result = Files::new();

        for file in files {
            let path = PathBuf::from(file);

            let item = File::from_name(BString::from(path.as_os_str().as_bytes()), path, flags);

            match item {
                Ok(item) => {
                    result.push(item);
                },
                Err(err) => {
                    eprintln!("ls: cannot access '{}': {}", file, err);
                    process::exit(1);
                },
            }
        }

        sort(&mut result, &flags);

        exit_code = output(result, &mut writer, flags);
    } else if flags.recursive {
        for file in files {
            exit_code = recursive_output(file, &mut writer, &flags);
        }
    } else {
        let multiple = files.len() > 1;

        for (i, file) in files.enumerate() {
            if multiple {
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

            let mut result = Files::new();

            let path = PathBuf::from(file);

            if path.is_file() {
                let item = File::from(PathBuf::from(file), flags);

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
                result = collect(file, &flags);
            }

            exit_code = output(result, &mut writer, flags);
        }
    }

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

/// Read the `&str` as a directory and collect the results into a `File` vector.
fn collect(file: &str, flags: &Flags) -> Files {
    let mut result = Files::new();

    match fs::read_dir(file) {
        Ok(dir) => {
            for entry in dir {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => {
                        eprintln!("ls: cannot access '{}': {}", file, err);
                        process::exit(1);
                    },
                };

                let file = match File::from(entry.path(), *flags) {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!("ls: cannot access '{}': {}", file, err);
                        process::exit(1);
                    },
                };

                if !File::is_hidden(file.name.as_bstr()) || flags.show_hidden() {
                    result.push(file);
                }
            }

            if !flags.no_sort {
                sort(&mut result, flags);
            }
        },
        Err(err) => {
            eprintln!("ls: cannot access '{}': {}", file, err);
            process::exit(1);
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

        let dot = File::from_name(BString::from("."), current.clone(), *flags);

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

        let dot_dot = File::from_name(BString::from(".."), PathBuf::from(parent_path), *flags);

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

    result
}

/// Recursively display sub directories from a given path.
fn recursive_output(file: &str, writer: &mut BufWriter<io::Stdout>, flags: &Flags) -> i32 {
    match writeln!(writer, "\n{}:", file) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("ls:  '{}'", err);
        },
    }

    let path = PathBuf::from(file);

    let files = if path.is_file() {
        let mut result = Files::new();
        let item = File::from(PathBuf::from(file), *flags);

        match item {
            Ok(item) => {
                result.push(item);
            },
            Err(err) => {
                eprintln!("ls: cannot access {}: {}", err, file);
            },
        };

        result
    } else {
        collect(file, flags)
    };
    let mut exit_code = output(files, writer, *flags);

    if path.is_file() {
        return exit_code;
    }

    match fs::read_dir(file) {
        Ok(dir) => {
            for entry in dir {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();

                        if path.is_dir() {
                            let file_string = path.to_string_lossy().to_string();

                            exit_code = recursive_output(&file_string, writer, flags);
                        }
                    },
                    Err(err) => {
                        eprintln!("ls: cannot access '{}': {}", file, err);
                    },
                };
            }
        },
        Err(err) => {
            eprintln!("ls: cannot access '{}': {}", file, err);
        },
    }

    exit_code
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
fn sort_by_access_time(file: &File) -> i64 {
    file.metadata.atime()
}

/// Sort a list of files by last change of file status information
fn sort_by_last_changed_time(file: &File) -> i64 {
    file.metadata.ctime()
}

/// Sort a list of files by file name alphabetically
fn sort_by_name(file: &File) -> String {
    file.name.to_string().to_lowercase().trim_start_matches('.').to_string()
}

/// Sort a list of files by size
fn sort_by_size(file: &File) -> u64 {
    file.metadata.len()
}

/// Sort a list of directories by modification time
fn sort_by_time(file: &File) -> i64 {
    file.metadata.mtime()
}
