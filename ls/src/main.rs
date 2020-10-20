use std::io::{self, Result, Write};
use std::string::String;
use std::time::SystemTime;
use std::{fs, process};

use pad::{Alignment, PadStr};

extern crate chrono;

mod cli;
mod file;
mod flags;

use file::File;
use flags::Flags;

fn main() {
    let matches = cli::create_app().get_matches();

    let files = matches.values_of("FILE").unwrap();
    let flags = Flags::from_matches(&matches);

    let mut exit_code = 0;

    let mut writer: Box<dyn Write> = Box::new(io::stdout());

    for file in files {
        match fs::read_dir(file) {
            Ok(dir) => {
                if flags.all {
                    todo!();
                }

                let mut dir: Vec<_> =
                    dir.map(|entry| File::from(entry.unwrap(), flags).unwrap()).collect();

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
                    if print_list(dir, &mut writer, flags).is_err() {
                        exit_code = 1
                    }
                } else if print_default(dir, &mut writer, flags).is_err() {
                    exit_code = 1;
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
fn print_default<W: Write>(files: Vec<File>, writer: &mut W, flags: Flags) -> Result<()> {
    for file in files {
        if File::is_hidden(&file.name) && !flags.show_hidden() {
            continue;
        }

        let file_name = file.get_file_name();

        if flags.comma_separate {
            write!(writer, "{}, ", file_name)?;
        } else {
            writeln!(writer, "{}", file_name)?;
        }
    }
    if flags.comma_separate {
        writeln!(writer)?;
    }

    Ok(())
}

/// Prints information about the provided file in a long format
fn print_list<W: Write>(files: Vec<File>, writer: &mut W, flags: Flags) -> Result<()> {
    let mut inode_width = 1;
    let mut block_width = 1;
    let mut hard_links_width = 1;
    let mut user_width = 1;
    let mut group_width = 1;
    let mut size_width = 1;

    for file in &files {
        if File::is_hidden(&file.name) && !flags.show_hidden() {
            continue;
        }

        if flags.inode {
            let inode = file.get_inode().len();

            if inode > inode_width {
                inode_width = inode;
            }
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
        if flags.inode {
            write!(
                writer,
                "{} ",
                file.get_inode().pad_to_width_with_alignment(inode_width, Alignment::Right)
            )?;
        }

        if flags.size {
            write!(
                writer,
                "{} ",
                file.get_blocks().pad_to_width_with_alignment(block_width, Alignment::Right)
            )?;
        }

        write!(writer, "{} ", file.get_permissions())?;

        write!(
            writer,
            "{} ",
            file.get_hard_links().pad_to_width_with_alignment(hard_links_width, Alignment::Right)
        )?;

        write!(writer, "{} ", file.get_user().pad_to_width(user_width))?;

        if !flags.no_owner {
            write!(writer, "{} ", file.get_group().pad_to_width(group_width))?;
        }

        write!(
            writer,
            "{} ",
            file.get_size().pad_to_width_with_alignment(size_width, Alignment::Right)
        )?;

        write!(writer, "{} ", file.get_time())?;

        write!(writer, "{}", file.get_file_name())?;

        writeln!(writer)?;
    }

    Ok(())
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
