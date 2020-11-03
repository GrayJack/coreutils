use std::{
    fs,
    io::{self, BufWriter, Write},
    os::unix::fs::MetadataExt,
    path, process,
    string::String,
    time::SystemTime,
};

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
                    }
                }
            }

            match writeln!(writer, "{}:", file) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("ls: {}", err);
                    process::exit(1);
                }
            }
        }

        let mut result: Vec<File>;

        let path = path::PathBuf::from(file);

        if path.is_file() {
            result = Vec::new();

            let item = File::from(path::PathBuf::from(file), flags);

            match item {
                Ok(item) => {
                    result.push(item);
                },
                Err(err) => {
                    eprintln!("ls: cannot access {}: {}", err, file);
                    process::exit(1);
                }
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

        if flags.all || flags.no_sort {
            // Retrieve the current directories information. This must
            // be canonicalize incase the path is relative
            let current = path::PathBuf::from(file).canonicalize();

            let current = match current {
                Ok(current) => current,
                Err(err) => {
                    eprintln!("ls: {}", err);
                    process::exit(1);
                }
            };

            let dot = File::from_name(".".to_string(), current.clone(), flags);

            let dot = match dot {
                Ok(dot) => dot,
                Err(err) => {
                    eprintln!("ls: {}", err);
                    process::exit(1);
                }
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
                }
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

#[derive(PartialEq, Eq)]
enum Alignment {
    Left,
    Right,
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

    pub fn width(&self) -> usize { unsafe { *self.width } }
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

    let mut total: u64 = 0;

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

        total += file.blocks();

        // Process the file's block size
        if flags.size {
            let block = file.blocks() as usize;
            let block_len = block.to_string().len();

            if block_len > block_width {
                block_width = block_len;
            }

            let block_width_ptr: *mut usize = &mut block_width;

            row.push(Column::from(block.to_string(), block_width_ptr, Alignment::Right));
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

        let user_width_ptr: *mut usize = &mut user_width;

        row.push(Column::from(user, user_width_ptr, Alignment::Left));

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

    writeln!(writer, "total {}", total)?;

    for row in rows {
        for column in row {
            if column.alignment == Alignment::Left {
                write!(writer, "{:<1$} ", column.value, column.width())?;
            } else if column.alignment == Alignment::Right {
                write!(writer, "{:>1$} ", column.value, column.width())?;
            }
        }

        writeln!(writer)?;
    }

    Ok(())
}

/// Sort a list of files by last accessed time
fn sort_by_access_time(file: &File) -> SystemTime {
    let accessed = file.metadata.accessed();

    match accessed {
        Ok(accessed) => accessed,
        Err(err) => {
            eprintln!("ls: {}", err);
            SystemTime::now()
        },
    }
}

/// Sort a list of files by file name alphabetically
fn sort_by_name(file: &File) -> String {
    file.name.to_lowercase().trim_start_matches('.').to_string()
}

/// Sort a list of files by size
fn sort_by_size(file: &File) -> u64 { file.metadata.len() }

/// Sort a list of directories by modification time
fn sort_by_time(file: &File) -> SystemTime {
    let modified = file.metadata.modified();

    match modified {
        Ok(modified) => modified,
        Err(err) => {
            eprintln!("ls: {}", err);
            SystemTime::now()
        },
    }
}
