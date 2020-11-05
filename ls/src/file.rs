use coreutils_core::os::{
    group::{Error as GroupError, Group},
    passwd::{Error as PasswdError, Passwd},
};

use std::{
    fs, io,
    os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt},
    path,
    result::Result,
    string::String,
};

use ansi_term::Color;

extern crate chrono;

use chrono::{DateTime, Local, TimeZone};

use crate::flags::Flags;

/// Represents a file and it's properties
pub(crate) struct File {
    pub name: String,
    pub path: path::PathBuf,
    pub metadata: fs::Metadata,
    flags: Flags,
}

impl File {
    /// Creates a `File` instance from a `DirEntry`
    pub fn from(path: path::PathBuf, flags: Flags) -> io::Result<Self> {
        let metadata = path.symlink_metadata()?;

        if flags.dereference && metadata.file_type().is_symlink() {
            let symlink = fs::read_link(path.clone())?;

            let name = File::path_buf_to_file_name(&symlink)?.to_string();

            let metadata = path.metadata()?;

            return Ok(File { name, path: symlink, metadata, flags });
        }

        let name = File::path_buf_to_file_name(&path)?.to_string();

        Ok(File { name, path, metadata, flags })
    }

    /// Creates a `File` instance from a `DirEntry` and supplies a file name
    pub fn from_name(name: String, path: path::PathBuf, flags: Flags) -> io::Result<Self> {
        let metadata = path.metadata()?;

        Ok(File { name, path, metadata, flags })
    }

    /// Retrieves the number of blocks allocated to a file as a string
    pub fn blocks(&self) -> u64 {
        let blocks = self.metadata.blocks();

        if self.flags.block_size {
            let st_size = blocks * 512;

            st_size / 1024
        } else {
            blocks
        }
    }

    /// Retrieves a files permissions as a string
    pub fn permissions(&self) -> String {
        let mode = self.metadata.permissions().mode();

        unix_mode::to_string(mode)
    }

    /// Retrieves the number of hard links pointing to a file as a string
    pub fn hard_links(&self) -> String { self.metadata.nlink().to_string() }

    /// Retrieves the inode number as a string
    pub fn inode(&self) -> String { self.metadata.ino().to_string() }

    /// Retrieves the file's user name as a string. If the `-n` flag is set,
    /// the the user's ID is returned
    pub fn user(&self) -> Result<String, PasswdError> {
        if self.flags.numeric_uid_gid {
            return Ok(self.metadata.uid().to_string());
        }

        match Passwd::from_uid(self.metadata.uid()) {
            Ok(passwd) => Ok(passwd.name().to_string()),
            Err(err) => Err(err),
        }
    }

    /// Retrieves the file's group name as a string. If the `-n` flag is set,
    /// the the group's ID is returned
    pub fn group(&self) -> Result<String, GroupError> {
        if self.flags.numeric_uid_gid {
            return Ok(self.metadata.gid().to_string());
        }

        match Group::from_gid(self.metadata.gid()) {
            Ok(group) => Ok(group.name().to_string()),
            Err(err) => Err(err),
        }
    }

    /// Retrieve the file's size, in bytes, as a string
    pub fn size(&self) -> String { self.metadata.len().to_string() }

    /// Retrieves the file's timestamp as a string
    ///
    /// By default the file's modified time is displayed. The `-u` flag will
    /// display the last accessed time. The `-c` flag will display the last
    /// modified time of the file's status information. The date format used is
    /// `%b %e %H:%M` unless the duration is greater than six months, which case
    /// the date format will be `%b %e  %Y`.
    pub fn time(&self) -> String {
        let (secs, nsecs) = if self.flags.last_accessed {
            // Retrieve the files last accessed time
            (self.metadata.atime(), self.metadata.atime_nsec())
        } else if self.flags.file_status_modification {
            // Retrieve the files last modification time of the status
            // information
            (self.metadata.ctime(), self.metadata.ctime_nsec())
        } else {
            // Retrieve the files modification time
            (self.metadata.mtime(), self.metadata.mtime_nsec())
        };

        let datetime: DateTime<Local> = Local.timestamp(secs, nsecs as u32);

        let mut fmt = "%b %e %H:%M";

        let now: DateTime<Local> = Local::now();

        let duration = datetime.signed_duration_since(now);

        let six_months = -182;

        if duration.num_days() < six_months {
            fmt = "%b %e  %Y";
        }

        datetime.format(fmt).to_string()
    }

    /// Check if a path is an executable file
    pub fn is_executable(path: &path::PathBuf) -> bool {
        let mut result = false;

        let metadata = fs::symlink_metadata(path);

        if let Ok(metadata) = metadata {
            result = metadata.is_file() && metadata.permissions().mode() & 0o111 != 0;
        }

        result
    }

    /// Checks if a string looks like a hidden unix file name
    pub fn is_hidden(name: &str) -> bool { name.starts_with('.') }

    /// Gets the file name from a `PathBuf`
    ///
    /// Will return `Error` if the path terminates at '..' or if the file name
    /// contains invalid unicode characters.
    pub fn path_buf_to_file_name(path: &path::PathBuf) -> io::Result<&str> {
        // Create a new IO Error.
        let io_error = |kind: io::ErrorKind, msg: &str| io::Error::new(kind, msg);

        let file_name = match path.file_name() {
            Some(file_name) => file_name,
            None => {
                return Err(io_error(io::ErrorKind::NotFound, "Path terminates at \"..\""));
            },
        };

        let file_name = match file_name.to_str() {
            Some(file_name) => file_name,
            None => {
                return Err(io_error(
                    io::ErrorKind::InvalidData,
                    "File name contains invalid unicode",
                ));
            },
        };

        Ok(file_name)
    }

    /// Gets a file name from a directory entry and adds appropriate formatting
    pub fn file_name(&self) -> String {
        let mut file_name = self.name.clone();

        let flags = self.flags;

        if File::is_executable(&self.path) {
            file_name = self.add_executable_color(file_name);

            if flags.classify {
                file_name = format!("{}*", file_name);
            }
        }

        if self.metadata.file_type().is_symlink() && !flags.dereference {
            file_name = self.add_symlink_color(file_name);

            if flags.classify && !flags.show_list() {
                file_name = format!("{}@", file_name);
            }

            if flags.show_list() {
                let symlink = fs::read_link(self.path.clone());

                if let Ok(symlink) = symlink {
                    let mut symlink_name = symlink.to_string_lossy().to_string();

                    if File::is_executable(&symlink) {
                        symlink_name = self.add_executable_color(symlink_name);

                        if flags.classify {
                            symlink_name = format!("{}*", symlink_name);
                        }
                    }

                    file_name = format!("{} -> {}", file_name, symlink_name);
                }
            }
        }

        if self.metadata.file_type().is_fifo() {
            file_name = self.add_named_pipe_color(file_name);

            if flags.classify {
                file_name = format!("{}|", file_name);
            }
        }

        if self.metadata.is_dir() {
            file_name = self.add_directory_color(file_name);

            if flags.classify || flags.indicator {
                file_name = format!("{}/", file_name);
            }
        }

        file_name
    }

    /// Adds a bold green color to a file name to represent an executable
    pub fn add_executable_color(&self, file_name: String) -> String {
        Color::Green.bold().paint(file_name).to_string()
    }

    /// Adds a bold blue color to a directory name
    pub fn add_directory_color(&self, directory_name: String) -> String {
        Color::Blue.bold().paint(directory_name).to_string()
    }

    pub fn add_named_pipe_color(&self, named_pipe_name: String) -> String {
        Color::Yellow.on(Color::Black).paint(named_pipe_name).to_string()
    }

    /// Adds a bold cyan color to a file name to represent a symlink
    pub fn add_symlink_color(&self, symlink_name: String) -> String {
        Color::Cyan.bold().paint(symlink_name).to_string()
    }
}
