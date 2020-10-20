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
                let mut dir: Vec<_> = dir.map(|entry| {
                    File::from(entry.unwrap(), flags).unwrap()
                }).collect();

                if flags.time {
                    if flags.last_accessed {
                        dir.sort_by_key(sort_by_access_time);
                    } else {
                        dir.sort_by_key(sort_by_time);
                    }
                    dir.reverse();
                } else if flags.sort_size {
                    dir.sort_by_key(sort_by_size);
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
fn print_default(files: Vec<File>, flags: LsFlags) -> i32 {
    let exit_code = 1;

    for file in files {
        if File::is_hidden(&file.name) && !flags.all {
            continue;
        }

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
fn print_list(files: Vec<File>, flags: LsFlags) -> i32 {
    let exit_code = 1;

    let mut block_width = 1;
    let mut hard_links_width = 1;
    let mut user_width = 1;
    let mut group_width = 1;
    let mut size_width = 1;

    for file in &files {
        if File::is_hidden(&file.name) && !flags.all {
            continue;
        }

        if flags.size {
            let block = file.get_blocks().len();

            if block > block_width {
                block_width = block;
            }
        }

        let hard_links = file.get_hard_links().len();

        if hard_links > hard_links_width {
            hard_links_width = hard_links;
        }

        let user = file.get_user().len();

        if user > user_width {
            user_width = user;
        }

        if !flags.no_owner {
            let group = file.get_group().len();

            if group > group_width {
                group_width = group;
            }
        }

        let size = file.get_size().len();

        if size > size_width {
            size_width = size;
        }
    }

    for file in &files {
        if flags.size {
            print!(
                "{} ",
                file.get_blocks().pad_to_width_with_alignment(block_width, Alignment::Right)
            );
        }

        print!("{} ", file.get_permissions());

        print!(
            "{} ",
            file.get_hard_links().pad_to_width_with_alignment(hard_links_width, Alignment::Right)
        );

        print!("{} ", file.get_user().pad_to_width(user_width));

        if !flags.no_owner {
            print!("{} ", file.get_group().pad_to_width(group_width));
        }

        print!("{} ", file.get_size().pad_to_width_with_alignment(size_width, Alignment::Right));

        print!("{} ", file.get_time());

        print!("{}", file.get_file_name());

        println!();
    }

    exit_code
}

/// Sort a list of files by last accessed time
fn sort_by_access_time(file: &File) -> SystemTime {
    let metadata = file.metadata.clone();

    metadata.accessed().unwrap()
}

/// Sort a list of files by file name alphabetically
fn sort_by_name(file: &File) -> String {
    file.name.to_lowercase()
}

/// Sort a list of files by size
fn sort_by_size(file: &File) -> u64 {
    file.metadata.len()
}

/// Sort a list of directories by modification time
fn sort_by_time(file: &File) -> SystemTime {
    file.metadata.modified().unwrap()
}
