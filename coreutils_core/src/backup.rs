use regex::Regex;
use std::{
    fs,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupMode {
    None,
    Numbered,
    Existing,
    Simple,
}

impl BackupMode {
    pub fn from_string(string: &str) -> BackupMode {
        match string {
            "none" | "off" => BackupMode::None,
            "numbered" | "t" => BackupMode::Numbered,
            "existing" | "nil" => BackupMode::Existing,
            "simple" | "never" => BackupMode::Simple,
            _ => BackupMode::Existing,
        }
    }
}

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
        return create_numbered_backup(file);
    } else {
        return create_simple_backup(file, suffix);
    }
}

pub fn create_simple_backup(file: &Path, suffix: &str) -> Result<PathBuf, Error> {
    let new = PathBuf::from(format!("{}{}", file.display(), suffix));

    match fs::rename(file, &new) {
        Ok(()) => Ok(new),
        Err(error) => Err(error),
    }
}
