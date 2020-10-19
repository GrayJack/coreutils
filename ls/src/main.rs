use std::string::String;
use std::time::SystemTime;
use std::{fs, process};

use pad::{Alignment, PadStr};

extern crate chrono;

mod cli;
mod file;
mod flags;

use file::File;
use flags::LsFlags;

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
                    dir.reverse();
                } else {
                    // Sort the directory entries by file name by default
                    dir.sort_by_key(sort_by_name);
                }

                if flags.reverse {
                    dir.reverse();
                }

                if !flags.comma_separate && flags.show_list() {
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
        if File::is_hidden(&entry) && !flags.all {
            continue;
        }

        let path = entry.path();

        let file = File::from(entry, fs::symlink_metadata(path).unwrap(), flags);

        let file_name = file.get_file_name();

        if flags.comma_separate {
            print!("{}, ", file_name);
        } else {
            println!("{}", file_name);
        }
    }
    if flags.comma_separate {
        println!();
    }

    exit_code
}

/// Prints information about the provided file in a long format
fn print_list(dir: Vec<fs::DirEntry>, flags: LsFlags) -> i32 {
    let mut exit_code = 1;

    let mut rows: Vec<File> = Vec::new();

    let mut block_width = 1;
    let mut hard_links_width = 1;
    let mut user_width = 1;
    let mut group_width = 1;
    let mut size_width = 1;

    for entry in dir {
        match fs::symlink_metadata(entry.path()) {
            Ok(metadata) => {
                if File::is_hidden(&entry) && !flags.all {
                    continue;
                }

                let row = File::from(entry, metadata, flags);

                if flags.size {
                    let block = row.get_blocks().len();

                    if block > block_width {
                        block_width = block;
                    }
                }

                let hard_links = row.get_hard_links().len();

                if hard_links > hard_links_width {
                    hard_links_width = hard_links;
                }

                let user = row.get_user().len();

                if user > user_width {
                    user_width = user;
                }

                if !flags.no_owner {
                    let group = row.get_group().len();

                    if group > group_width {
                        group_width = group;
                    }
                }

                let size = row.get_size().len();

                if size > size_width {
                    size_width = size;
                }

                rows.push(row);
            }
            Err(err) => {
                eprintln!("ls: {}", err);
                exit_code = 1;
            }
        }
    }

    for row in &rows {
        if flags.size {
            print!(
                "{} ",
                row.get_blocks().pad_to_width_with_alignment(block_width, Alignment::Right)
            );
        }

        print!("{} ", row.get_permissions());

        print!(
            "{} ",
            row.get_hard_links().pad_to_width_with_alignment(hard_links_width, Alignment::Right)
        );

        print!("{} ", row.get_user().pad_to_width(user_width));

        if !flags.no_owner {
            print!("{} ", row.get_group().pad_to_width(group_width));
        }

        print!("{} ", row.get_size().pad_to_width_with_alignment(size_width, Alignment::Right));

        print!("{} ", row.get_time());

        print!("{}", row.get_file_name());

        println!();
    }

    exit_code
}

/// Sort a list of directories by file name alphabetically
fn sort_by_name(dir: &fs::DirEntry) -> String {
    let file_name = dir.file_name().into_string().unwrap();

    file_name.to_lowercase()
}

/// Sort a list of directories by modification time
fn sort_by_time(dir: &fs::DirEntry) -> SystemTime {
    let metadata = fs::metadata(dir.path());

    if let Ok(metadata) = metadata {
        metadata.modified().unwrap()
    } else {
        SystemTime::now()
    }
}
