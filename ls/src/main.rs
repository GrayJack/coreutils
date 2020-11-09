use std::{
    fs,
    io::{self, BufWriter, Write},
    os::unix::fs::MetadataExt,
    path, process,
    string::String,
};

use coreutils_core::os::tty::{is_tty, tty_dimensions};

use term_grid::{Alignment, Cell, Direction, Filling, Grid, GridOptions};

extern crate chrono;

mod cli;
mod file;
mod flags;
mod table;

use file::{File, FileColor};
use flags::Flags;
use table::{Row, Table};

fn main() {
    let matches = cli::create_app().get_matches();

    let files = matches.values_of("FILE").unwrap();
    let flags = Flags::from_matches(&matches);

    let mut exit_code = 0;

    let mut writer = BufWriter::new(io::stdout());

    let multiple = files.len() > 1;

    for (i, file) in files.enumerate() {
        if multiple {
            if i == 0 {
                match writeln!(writer, "\n") {
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

        let mut result: Vec<File>;

        let path = path::PathBuf::from(file);

        if flags.directory || path.is_file() {
            result = Vec::new();

            let item = if flags.directory {
                File::from_name(path.to_string_lossy().to_string(), path, flags)
            } else {
                File::from(path::PathBuf::from(file), flags)
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
                        .filter(|file| !File::is_hidden(&file.name) || flags.show_hidden())
                        .collect();

                    if !flags.no_sort {
                        if flags.time {
                            if flags.last_accessed {
                                result.sort_by_key(sort_by_access_time);
                            } else if flags.file_status_modification {
                                result.sort_by_key(sort_by_ctime)
                            } else {
                                result.sort_by_key(sort_by_time);
                            }
                            result.reverse();
                        } else if flags.sort_size {
                            result.sort_by_key(sort_by_size);
                            result.reverse();
                        } else {
                            // Sort the directory entries by file name by default
                            result.sort_by_key(sort_by_name);
                        }

                        if flags.reverse {
                            result.reverse();
                        }
                    }
                },
                Err(err) => {
                    eprintln!("ls: cannot access '{}': {}", file, err);
                    exit_code = 1;

                    break;
                },
            }
        }

        if !flags.directory && (flags.all || flags.no_sort) {
            // Retrieve the current directories information. This must
            // be canonicalize incase the path is relative
            let current = path::PathBuf::from(file).canonicalize();

            let current = match current {
                Ok(current) => current,
                Err(err) => {
                    eprintln!("ls: {}", err);
                    process::exit(1);
                },
            };

            let dot = File::from_name(".".to_string(), current.clone(), flags);

            let dot = match dot {
                Ok(dot) => dot,
                Err(err) => {
                    eprintln!("ls: {}", err);
                    process::exit(1);
                },
            };

            // Retrieve the parent path. Default to the current path if the parent doesn't
            // exist
            let parent_path =
                path::PathBuf::from(dot.path.parent().unwrap_or_else(|| current.as_path()));

            let dot_dot = File::from_name("..".to_string(), parent_path, flags);

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

        if !flags.comma_separate && flags.show_list() {
            match print_list(result, &mut writer, flags) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("ls: cannot access '{}': {}", file, err);
                    exit_code = 1;
                },
            }
        } else {
            match print_default(result, &mut writer, flags) {
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

/// Prints information about a file in the default format
fn print_default<W: Write>(files: Vec<File>, writer: &mut W, flags: Flags) -> io::Result<()> {
    if !is_tty(&io::stdout()) {
        for file in &files {
            writeln!(writer, "{}", file.name)?;
        }

        return Ok(());
    } else if flags.comma_separate {
        for (i, file) in files.iter().enumerate() {
            let file_name = &file.name;

            if (i + 1) == files.len() {
                writeln!(writer, "{}", file_name)?;
            } else {
                write!(writer, "{}, ", file_name)?;
            }
        }

        return Ok(());
    } else if flags.order_left_to_right && !flags.order_top_to_bottom {
        return print_grid(files, writer, Direction::LeftToRight);
    }

    print_grid(files, writer, Direction::TopToBottom)
}

fn print_grid<W: Write>(files: Vec<File>, writer: &mut W, direction: Direction) -> io::Result<()> {
    let io_error = |kind: io::ErrorKind, msg: &str| io::Error::new(kind, msg);

    let mut grid = Grid::new(GridOptions { filling: Filling::Spaces(2), direction });

    let width = match tty_dimensions(&io::stdout()) {
        Some(result) => result.0,
        None => {
            return Err(io_error(io::ErrorKind::Other, "Unable to retrieve terminal dimensions."));
        },
    };

    for file in &files {
        grid.add(Cell {
            alignment: Alignment::Left,
            contents:  file.file_name(FileColor::Show),
            width:     file.file_name(FileColor::Hide).len(),
        });
    }

    match grid.fit_into_width(width.into()) {
        Some(display) => {
            write!(writer, "{}", display)?;
            Ok(())
        },
        None => {
            for file in &files {
                writeln!(writer, "{}", file.file_name(FileColor::Show))?;
            }

            Ok(())
        },
    }
}

/// Prints information about the provided file in the long (`-l`) format
fn print_list<W: Write>(files: Vec<File>, writer: &mut W, flags: Flags) -> io::Result<()> {
    let mut inode_width = 1;
    let mut block_width = 1;
    let permissions_width = 1;
    let mut hard_links_width = 1;
    let mut user_width = 1;
    let mut group_width = 1;
    let mut size_width = 1;
    let time_width = 1;
    let file_name_width = 1;

    let mut rows = Table::new();

    let mut total: u64 = 0;

    for file in &files {
        let mut row = Row::new();

        // Process the file's inode
        if flags.inode {
            let inode = file.inode();
            let inode_len = inode.len();

            if inode_len > inode_width {
                inode_width = inode_len;
            }

            row.inode = inode;
        }

        total += file.blocks();

        // Process the file's block size
        if flags.size {
            let block = file.blocks() as usize;
            let block_len = block.to_string().len();

            if block_len > block_width {
                block_width = block_len;
            }

            row.block = block.to_string();
        }

        // Process the file's permissions
        let permissions = file.permissions();

        row.permissions = permissions;

        // Process the file's hard links
        let hard_links = file.hard_links();
        let hard_links_len = hard_links.len();

        if hard_links_len > hard_links_width {
            hard_links_width = hard_links_len;
        }

        row.hard_links = hard_links;

        // Process the file's user name
        let user = match file.user() {
            Ok(file_user) => file_user,
            Err(err) => {
                eprintln!("ls: {}", err);
                file.metadata.uid().to_string()
            },
        };

        let user_len = user.len();

        if user_len > user_width {
            user_width = user_len;
        }

        row.user = user;

        // Process the file's group name
        if !flags.no_owner {
            let group = match file.group() {
                Ok(file_group) => file_group,
                Err(err) => {
                    eprintln!("ls: {}", err);
                    file.metadata.gid().to_string()
                },
            };

            let group_len = group.len();

            if group_len > group_width {
                group_width = group_len;
            }

            row.group = group;
        }

        // Process the file's size
        let size = file.size();
        let size_len = file.size().len();

        if size_len > size_width {
            size_width = size_len;
        }

        row.size = size;

        // Process the file's timestamp
        row.time = file.time();

        // Process the file's name
        row.file_name = file.file_name(FileColor::Show);

        rows.push(row);
    }

    writeln!(writer, "total {}", total)?;

    for row in rows {
        if flags.inode {
            write!(writer, "{:>1$} ", row.inode, inode_width)?;
        }

        if flags.size {
            write!(writer, "{:>1$} ", row.block, block_width)?;
        }

        write!(writer, "{:<1$} ", row.permissions, permissions_width)?;

        write!(writer, "{:>1$} ", row.hard_links, hard_links_width)?;

        write!(writer, "{:<1$} ", row.user, user_width)?;

        if !flags.no_owner {
            write!(writer, "{:<1$} ", row.group, group_width)?;
        }

        write!(writer, "{:>1$} ", row.size, size_width)?;

        write!(writer, "{:<1$} ", row.time, time_width)?;

        write!(writer, "{:<1$} ", row.file_name, file_name_width)?;

        writeln!(writer)?;
    }

    Ok(())
}

/// Sort a list of files by last accessed time
fn sort_by_access_time(file: &File) -> i64 { file.metadata.atime() }

/// Sort a list of files by last change of file status information
fn sort_by_ctime(file: &File) -> i64 { file.metadata.ctime() }

/// Sort a list of files by file name alphabetically
fn sort_by_name(file: &File) -> String {
    file.name.to_lowercase().trim_start_matches('.').to_string()
}

/// Sort a list of files by size
fn sort_by_size(file: &File) -> u64 { file.metadata.len() }

/// Sort a list of directories by modification time
fn sort_by_time(file: &File) -> i64 { file.metadata.mtime() }
