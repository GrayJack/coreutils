//! The Backup module handles creating backups using the methods
//! specified in the GNU coreutils manual.
//!
//! ## About
//! The GNU coreutils [backup options](https://www.gnu.org/software/coreutils/manual/html_node/Backup-options.html#Backup-options)
//! support 4 backup modes:
//! - None
//! - Numbered
//! - Existing
//! - Simple
//!
//! **NOTE:** This module does not figure out default values based on environment
//! variables as defined in the GNU backup options manual page. Whether to adhere to the
//! GNU standard or not is up to the user of this module.
//!
//! ### None
//! Can be specified by either supplying the strings `none` or `off`.
//! Fairly self-explanatory: Never make backups.
//!
//! ### Numbered
//! Can be specified by either supplying the strings `numbered` or `t`.
//! This mode always makes numbered backups. This means that a backup of a file `a.txt`
//! will be backed up to `a.txt~X~` where `X` is the next number backup.
//!
//! For example, if we create a file named `main.rs` and then back it up in this mode, we
//! will get `main.rs~1~`. If we back the file up a second time, we will get `main.rs~2~`.
//!
//! ### Simple
//! Can be specified by either supplying the strings `simple` or `never` (not to be
//! confused with `none`) This mode simple appends a static suffix to the backup (This is
//! how Emacs makes backup files by default).
//!
//! For exmaple, if we create a file named `main.rs` and then back it up in this mode, we
//! will get `main.rs~`. If we back the file up a second time, we will overwrite the old
//! backup and the new backup will be called `main.rs~`.
//!
//! ### Existing
//! Can be specified by either supplying the strings `existing` or `nil`.
//! This mode checks for the existance of previous backups in any mode. If it finds
//! numbered backups, it will continue to make numbered backups. If it finds simple
//! backups, it will continue to make simple backups.

use regex::Regex;
use std::{
    fs,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

/// Convenience Enum to represent the different backup modes. See module documentation for
/// an in-depth overview of what each backup mode means/does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupMode {
    /// No backups will be made.
    None,
    /// Backups will be made of the form `<filename>~<X>~` where `X` is the next backup
    /// number.
    Numbered,
    /// The backup method will be consistant with what already exists for the current
    /// file.
    Existing,
    /// Backups will be made of the form `<filename><suffix>` where `suffix` is any
    /// suffix.
    Simple,
}

impl BackupMode {
    /// Creates an instance of `BackupMode` from a string slice. Any invalid input will
    /// result in `BackupMode::Existing` to be returned.
    pub fn from_string(string: impl AsRef<str>) -> Self { Self::from(string.as_ref()) }
}

impl From<&str> for BackupMode {
    /// Creates an instance of `BackupMode` from a string slice. Any invalid input will
    /// result in `BackupMode::Existing` to be returned.
    fn from(string: &str) -> Self {
        match string {
            "none" | "off" => BackupMode::None,
            "numbered" | "t" => BackupMode::Numbered,
            "existing" | "nil" => BackupMode::Existing,
            "simple" | "never" => BackupMode::Simple,
            _ => BackupMode::Existing,
        }
    }
}

/// Creates a numbered backup. Does so by taking the input `file` and poking the parent
/// directory to find a file of the form `<file>~<X>~` where `X` is a number. If none can
/// be found, a backup file is created where `X` is `1`. Else, it creates a backup file
/// where `X` is `X + 1`.
///
/// # Arguments
/// * `file` - the file to be backed up
/// # Remarks
/// Returns a `Result` of either a `PathBuf` to the newly created backup file or an
/// `io::Error`
pub fn create_numbered_backup(file: &Path) -> Result<PathBuf, Error> {
    let mut index = 1_u64;
    loop {
        if index == std::u64::MAX {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                "Cannot create backup: too many backup files",
            ));
        }

        let new = file.with_extension(format!("~{}~", index));
        if !new.exists() {
            match fs::rename(file, &new) {
                Ok(()) => return Ok(new),
                Err(err) => return Err(err),
            };
        }

        index += 1;
    }
}

/// Creates a backup in-keeping with previous backups. Pokes the directory to see whether
/// there are any numbered or simple backups of the input `file`. If numbered backups are
/// found, a numbered backup will be created. Else, a simple backup is created using the
/// input `suffix`
///
/// # Arguments
/// * `file` - the file to be backed up
/// * `suffix` - the suffix of the backup file
/// # Remarks
/// Returns a `Result` of either a `PathBuf` to the newly created backup file or an
/// `io::Error`
pub fn create_existing_backup(file: &Path, suffix: &str) -> Result<PathBuf, Error> {
    let mut has_numbered_backup = false;
    let regex = Regex::new(r"~\d+~").unwrap();
    let parent = file.parent().unwrap();
    for entry in parent.read_dir().unwrap() {
        if let Ok(entry) = entry {
            if let Some(ext) = entry.path().extension() {
                if regex.is_match(ext.to_str().unwrap()) {
                    has_numbered_backup = true;
                    break;
                }
            }
        }
    }

    if has_numbered_backup {
        create_numbered_backup(file)
    } else {
        create_simple_backup(file, suffix)
    }
}

/// Creates a simple backup. Creates a backup of the form `<file><suffix>`. Overwrites any
/// previous backup files with that same suffix.
///
/// # Arguments
/// * `file` - the file to be backed up
/// * `suffix` - the suffix of the backup file
/// # Remarks
/// Returns a `Result` of either a `PathBuf` to the newly created backup file or an
/// `io::Error`
pub fn create_simple_backup(file: &Path, suffix: &str) -> Result<PathBuf, Error> {
    let new = PathBuf::from(format!("{}{}", file.display(), suffix));

    match fs::rename(file, &new) {
        Ok(()) => Ok(new),
        Err(error) => Err(error),
    }
}
