use coreutils_core::os::group::Group;
use coreutils_core::os::passwd::Passwd;

use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::string::String;
use std::{fs, path};

use ansi_term::Color;

extern crate chrono;

use chrono::prelude::{DateTime, Local};

use crate::flags::LsFlags;

pub(crate) struct File {
    entry: fs::DirEntry,
    metadata: fs::Metadata,
    flags: LsFlags,
}

impl File {
    pub fn from(entry: fs::DirEntry, metadata: fs::Metadata, flags: LsFlags) -> Self {
        File { entry, metadata, flags }
    }

    /// Retrieves the number of blocks allocated to a file as a string
    pub fn get_blocks(&self) -> String {
        self.metadata.blocks().to_string()
    }

    /// Retrieves a files permissions as a string
    pub fn get_permissions(&self) -> String {
        let mode = self.metadata.permissions().mode();

        unix_mode::to_string(mode)
    }

    /// Retrieves the number of hard links pointing to a file as a string
    pub fn get_hard_links(&self) -> String {
        self.metadata.nlink().to_string()
    }

    /// Retrieves the file's user name as a string. If the `-n` flag is set,
    /// the the user's ID is returned
    pub fn get_user(&self) -> String {
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
    pub fn get_group(&self) -> String {
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
    pub fn get_size(&self) -> String {
        self.metadata.len().to_string()
    }

    /// Retrieves the file's timestamp as a string
    pub fn get_time(&self) -> String {
        let datetime: DateTime<Local>;

        if self.flags.last_accessed {
            let accessed = self.metadata.accessed().unwrap();
            datetime = accessed.into();
        } else {
            let modified = self.metadata.modified().unwrap();
            datetime = modified.into();
        }

        datetime.format("%b %e %k:%M").to_string()
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

    pub fn is_hidden(entry: &fs::DirEntry) -> bool {
        let mut result = false;

        let file_name = entry.file_name().into_string();

        if let Ok(file_name) = file_name {
            result = file_name.starts_with('.')
        }

        result
    }

    /// Gets a file name from a directory entry and adds appropriate formatting
    pub fn get_file_name(&self) -> String {
        let mut file_name = self.entry.file_name().into_string().unwrap();

        let flags = self.flags;

        let metadata = fs::symlink_metadata(self.entry.path());

        if let Ok(metadata) = metadata {
            if File::is_executable(&self.entry.path()) {
                file_name = self.add_executable_color(file_name);

                if flags.classify {
                    file_name = format!("{}*", file_name);
                }
            }

            if metadata.file_type().is_symlink() {
                file_name = self.add_symlink_color(file_name);

                if flags.classify && !flags.show_list() {
                    file_name = format!("{}@", file_name);
                }

                if flags.show_list() {
                    let symlink = fs::read_link(self.entry.path());

                    if let Ok(symlink) = symlink {
                        let mut symlink_name = String::from(symlink.to_str().unwrap());

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

            if metadata.file_type().is_fifo() {
                file_name = self.add_named_pipe_color(file_name);

                if flags.classify {
                    file_name = format!("{}|", file_name);
                }
            }

            if metadata.is_dir() {
                file_name = self.add_directory_color(file_name);

                if flags.classify {
                    file_name = format!("{}/", file_name);
                }
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