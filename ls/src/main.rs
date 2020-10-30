use std::{
    fs,
    io::{self, Write},
    os::unix::fs::MetadataExt,
    path, process,
    string::String,
    time::SystemTime,
};

use pad::{Alignment, PadStr};

extern crate chrono;

mod cli;
mod file;
mod flags;

use file::File;
use flags::Flags;

fn main() -> io::Result<()> {
    let matches = cli::create_app().get_matches();

    let files = matches.values_of("FILE").unwrap();
    let flags = Flags::from_matches(&matches);

    let mut exit_code = 0;

    let mut writer: Box<dyn Write> = Box::new(io::stdout());

    let multiple = files.len() > 1;

    for file in files {
        match fs::read_dir(file) {
            Ok(dir) => {
                let mut dir: Vec<_> = dir
                    // Collect information about the file or directory
                    .map(|entry| File::from(entry.unwrap().path(), flags).unwrap())
                    // Hide hidden files and directories if `-a` or `-A` flags
                    // weren't provided
                    .filter(|file| !File::is_hidden(&file.name) || flags.show_hidden())
                    .collect();

                if !flags.no_sort {
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
                }

                if flags.all || flags.no_sort {
                    // Retrieve the current directories information. This must
                    // be canonicalize incase the path is relative
                    let current = path::PathBuf::from(file).canonicalize().unwrap();

                    let dot = File::from_name(".".to_string(), current.clone(), flags)?;

                    // Retrieve the parent path. Default to the current path if the parent doesn't
                    // exist
                    let parent_path =
                        path::PathBuf::from(dot.path.parent().unwrap_or_else(|| current.as_path()));

                    let dot_dot = File::from_name("..".to_string(), parent_path, flags)?;

                    dir.insert(0, dot);
                    dir.insert(1, dot_dot);
                }

                if multiple {
                    writeln!(writer, "\n{}:", file)?;
                }

                if !flags.comma_separate && flags.show_list() {
                    match print_list(dir, &mut writer, flags) {
                        Ok(_) => {},
                        Err(err) => {
                            eprintln!("ls: cannot access '{}': {}", file, err);
                            exit_code = 1;
                        },
                    }
                } else {
                    match print_default(dir, &mut writer, flags) {
                        Ok(_) => {},
                        Err(err) => {
                            eprintln!("ls: cannot access '{}': {}", file, err);
                            exit_code = 1;
                        },
                    }
                }
            },
            Err(err) => {
                eprintln!("ls: cannot access '{}': {}", file, err);
                exit_code = 1;
            },
        }
    }

    if exit_code != 0 {
        process::exit(exit_code);
    }

    Ok(())
}

/// Prints information about a file in the default format
fn print_default<W: Write>(files: Vec<File>, writer: &mut W, flags: Flags) -> io::Result<()> {
    for file in files {
        let file_name = file.file_name();

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

struct Column {
    pub alignment: Alignment,
    width: *mut usize,
    pub value: String,
}

impl Column {
    pub fn from(value: String, width: *mut usize, alignment: Alignment) -> Self {
        Column { alignment, width, value }
    }

    pub fn width(&self) -> usize {
        unsafe {
            *self.width
        }
    }
}

/// Prints information about the provided file in the long (`-l`) format
fn print_list<W: Write>(files: Vec<File>, writer: &mut W, flags: Flags) -> io::Result<()> {
    let mut inode_width = 1;
    let mut block_width = 1;
    let mut permissions_width = 1;
    let mut hard_links_width = 1;
    let mut user_width = 1;
    let mut group_width = 1;
    let mut size_width = 1;
    let mut time_width = 1;
    let mut file_name_width = 1;

    let mut rows: Vec<Vec<Column>> = Vec::new();

    for file in &files {
        let mut row: Vec<Column> = Vec::new();

        // Process the file's inode
        if flags.inode {
            let inode = file.inode();
            let inode_len = inode.len();

            if inode_len > inode_width {
                inode_width = inode_len;
            }

            let inode_width_ptr: *mut usize = &mut inode_width;

            row.push(Column::from(inode, inode_width_ptr, Alignment::Right));
        }

        // Process the file's block size
        if flags.size {
            let block = file.blocks();
            let block_len = block.len();

            if block_len > block_width {
                block_width = block_len;
            }

            let block_width_ptr: *mut usize = &mut block_width;

            row.push(Column::from(block, block_width_ptr, Alignment::Right));
        }

        // Process the file's permissions
        let permissions = file.permissions();

        let permissions_width_ptr: *mut usize = &mut permissions_width;

        row.push(Column::from(permissions, permissions_width_ptr, Alignment::Left));

        // Process the file's hard links
        let hard_links = file.hard_links();
        let hard_links_len = hard_links.len();

        if hard_links_len > hard_links_width {
            hard_links_width = hard_links_len;
        }

        let hard_links_width_ptr: *mut usize = &mut hard_links_width;

        row.push(Column::from(hard_links, hard_links_width_ptr, Alignment::Right));

        // Process the file's user name
        let user = match file.user() {
            Ok(file_user) => {
                file_user
            },
            Err(err) => {
                eprintln!("ls: {}", err);
                file.metadata.uid().to_string()
            },
        };

        let user_len = user.len();

        if user_len > user_width {
            user_width = user_len;
        }

        let user_width_ptr: *mut usize = &mut user_width;

        row.push(Column::from(user, user_width_ptr, Alignment::Left));

        // Process the file's group name
        if !flags.no_owner {
            let group = match file.group() {
                Ok(file_group) => {
                    file_group
                },
                Err(err) => {
                    eprintln!("ls: {}", err);
                    file.metadata.gid().to_string()
                },
            };

            let group_len = group.len();

            if group_len > group_width {
                group_width = group_len;
            }

            let group_width_ptr: *mut usize = &mut group_width;

            row.push(Column::from(group, group_width_ptr, Alignment::Left));
        }

        // Process the file's size
        let size = file.size();
        let size_len = file.size().len();

        if size_len > size_width {
            size_width = size_len;
        }

        let size_width_ptr: *mut usize = &mut size_width;

        row.push(Column::from(size, size_width_ptr, Alignment::Right));

        // Process the file's timestamp
        let time_width_ptr: *mut usize = &mut time_width;

        row.push(Column::from(file.time()?, time_width_ptr, Alignment::Left));

        // Process the file's name
        let file_name_width_ptr: *mut usize = &mut file_name_width;

        row.push(Column::from(file.file_name(), file_name_width_ptr, Alignment::Left));

        rows.push(row);
    }

    for row in rows {
        for column in row {
            write!(
                writer,
                "{} ",
                column.value.pad_to_width_with_alignment(column.width(), column.alignment)
            )?;
        }

        writeln!(writer)?;
    }

    Ok(())
}

/// Sort a list of files by last accessed time
fn sort_by_access_time(file: &File) -> SystemTime { file.metadata.accessed().unwrap() }

/// Sort a list of files by file name alphabetically
fn sort_by_name(file: &File) -> String { file.name.to_lowercase() }

/// Sort a list of files by size
fn sort_by_size(file: &File) -> u64 { file.metadata.len() }

/// Sort a list of directories by modification time
fn sort_by_time(file: &File) -> SystemTime { file.metadata.modified().unwrap() }
