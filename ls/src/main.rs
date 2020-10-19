use clap::ArgMatches;

use coreutils_core::os::group::Group;
use coreutils_core::os::passwd::Passwd;

use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::string::String;
use std::time::SystemTime;
use std::{fs, path, process};

use ansi_term::Color;

use pad::{Alignment, PadStr};

extern crate chrono;

use chrono::prelude::{DateTime, Local};

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
        let file_name = get_file_name(&entry, flags);

        if is_hidden(&entry) && !flags.all {
            continue;
        }

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
                if is_hidden(&entry) && !flags.all {
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

/// Gets a file name from a directory entry and adds appropriate formatting
fn get_file_name(file: &fs::DirEntry, flags: LsFlags) -> String {
    let mut file_name = file.file_name().into_string().unwrap();

    let metadata = fs::symlink_metadata(file.path());

    if let Ok(metadata) = metadata {
        if is_executable(&file.path()) {
            file_name = add_executable_color(file_name);

            if flags.classify {
                file_name = format!("{}*", file_name);
            }
        }

        if metadata.file_type().is_symlink() {
            file_name = add_symlink_color(file_name);

            if flags.classify && !flags.show_list() {
                file_name = format!("{}@", file_name);
            }

            if flags.show_list() {
                let symlink = fs::read_link(file.path());

                if let Ok(symlink) = symlink {
                    let mut symlink_name = String::from(symlink.to_str().unwrap());

                    if is_executable(&symlink) {
                        symlink_name = add_executable_color(symlink_name);

                        if flags.classify {
                            symlink_name = format!("{}*", symlink_name);
                        }
                    }

                    file_name = format!("{} -> {}", file_name, symlink_name);
                }
            }
        }

        if metadata.file_type().is_fifo() {
            file_name = add_named_pipe_color(file_name);

            if flags.classify {
                file_name = format!("{}|", file_name);
            }
        }

        if metadata.is_dir() {
            file_name = add_directory_color(file_name);

            if flags.classify {
                file_name = format!("{}/", file_name);
            }
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

fn add_named_pipe_color(named_pipe_name: String) -> String {
    Color::Yellow.on(Color::Black).paint(named_pipe_name).to_string()
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
fn is_hidden(entry: &fs::DirEntry) -> bool {
    let mut result = false;

    let file_name = entry.file_name().into_string();

    if let Ok(file_name) = file_name {
        result = file_name.starts_with('.')
    }

    result
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

#[derive(Default, Copy, Clone)]
struct LsFlags {
    all: bool,
    classify: bool,
    comma_separate: bool,
    list: bool,
    no_owner: bool,
    numeric_uid_gid: bool,
    reverse: bool,
    size: bool,
    time: bool,
}

impl LsFlags {
    fn from_matches(matches: &ArgMatches<'_>) -> Self {
        let all = matches.is_present("all");
        let classify = matches.is_present("classify");
        let comma_separate = matches.is_present("comma_separate");
        let list = matches.is_present("list");
        let no_owner = matches.is_present("no_owner");
        let numeric_uid_gid = matches.is_present("numeric_uid_gid");
        let reverse = matches.is_present("reverse");
        let size = matches.is_present("size");
        let time = matches.is_present("time");

        LsFlags {
            all,
            classify,
            comma_separate,
            list,
            no_owner,
            numeric_uid_gid,
            reverse,
            size,
            time,
        }
    }

    /// Whether to print as a list based ont the provided flags
    fn show_list(&self) -> bool {
        !self.comma_separate && self.list || self.no_owner || self.numeric_uid_gid
    }
}

struct File {
    entry: fs::DirEntry,
    metadata: fs::Metadata,
    flags: LsFlags,
}

impl File {
    fn from(entry: fs::DirEntry, metadata: fs::Metadata, flags: LsFlags) -> Self {
        File { entry, metadata, flags }
    }

    /// Retrieves the number of blocks allocated to a file as a string
    fn get_blocks(&self) -> String {
        let blocks_value = self.metadata.blocks();
        let blocks: String = blocks_value.to_string();

        blocks
    }

    /// Retrieves a files permissions as a string
    fn get_permissions(&self) -> String {
        let mode = self.metadata.permissions().mode();

        unix_mode::to_string(mode)
    }

    /// Retrieves the number of hard links pointing to a file as a string
    fn get_hard_links(&self) -> String {
        let hard_links_value = self.metadata.nlink();
        let hard_links: String = hard_links_value.to_string();

        hard_links
    }

    /// Retrieves the file's user name as a string. If the `-n` flag is set,
    /// the the user's ID is returned
    fn get_user(&self) -> String {
        let user: String;

        if self.flags.numeric_uid_gid {
            let user_value = self.metadata.uid();
            user = user_value.to_string();
        } else {
            let uid = Passwd::from_uid(self.metadata.uid()).unwrap();
            let user_value = uid.name();
            user = user_value.to_string();
        }

        user
    }

    /// Retrieves the file's group name as a string. If the `-n` flag is set,
    /// the the group's ID is returned
    fn get_group(&self) -> String {
        let group: String;

        if self.flags.numeric_uid_gid {
            let group_value = self.metadata.gid();
            group = group_value.to_string();
        } else {
            let gid = Group::from_gid(self.metadata.gid()).unwrap();
            let group_value = gid.name();
            group = group_value.to_string();
        }

        group
    }

    /// Retrieve the file's size, in bytes, as a string
    fn get_size(&self) -> String {
        let size_value = self.metadata.len();
        let size: String = size_value.to_string();

        size
    }

    /// Retrieves the file's timestamp as a string
    fn get_time(&self) -> String {
        let modified = self.metadata.modified().unwrap();
        let modified_datetime: DateTime<Local> = modified.into();

        modified_datetime.format("%b %e %k:%M").to_string()
    }

    /// Retrieves the file's name and any terminal styling as a string
    fn get_file_name(&self) -> String {
        get_file_name(&self.entry, self.flags)
    }
}
