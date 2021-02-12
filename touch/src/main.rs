use std::{
    fs::{self, File, Metadata},
    process,
    time::SystemTime,
};

use clap::ArgMatches;
use coreutils_core::time::PrimitiveDateTime;
use filetime::{set_file_atime, set_file_mtime, set_file_times, set_symlink_file_times, FileTime};

mod cli;
// TODO: add Unit tests for touch
#[cfg(test)]
mod tests;

fn main() {
    let matches = cli::create_app().get_matches();

    let flags = TouchFlags::from_matches(&matches);

    // get files list in argument
    // Required argument, ok to unwrap and not check if is supplied.
    let files = matches.values_of("FILE").unwrap();

    touch(&files.collect::<Vec<_>>(), flags);
}

fn touch(files: &[&str], flags: TouchFlags) {
    let (new_atime, new_mtime) = new_filetimes(flags).unwrap_or_else(|err| {
        eprintln!("touch: {}", err);
        process::exit(1);
    });

    for filename in files {
        // if file already exist in the current directory
        let mut file_metadata =
            if flags.no_deref { fs::symlink_metadata(&filename) } else { fs::metadata(&filename) };

        if file_metadata.is_err() && !flags.no_create {
            match File::create(&filename) {
                Ok(_) => {
                    file_metadata = if flags.no_deref {
                        fs::symlink_metadata(&filename)
                    } else {
                        fs::metadata(&filename)
                    };
                },
                Err(e) => eprintln!("touch: Failed to create file {}: {}", &filename, e),
            }
        }
        if let Ok(file_metadata) = file_metadata {
            update_time(&filename, new_atime, new_mtime, &file_metadata, flags);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TouchFlags<'a> {
    access_time: bool,
    mod_time: bool,
    no_create: bool,
    no_deref: bool,
    reference_path: Option<&'a str>,
    date: Option<&'a str>,
    timestamp: Option<&'a str>,
}

impl<'a> TouchFlags<'a> {
    fn from_matches(matches: &'a ArgMatches<'a>) -> Self {
        let time_val = matches.value_of("time").unwrap_or("");
        let mut access_time = matches.is_present("accesstime")
            || time_val == "access"
            || time_val == "atime"
            || time_val == "use";

        let mut mod_time =
            matches.is_present("modification") || time_val == "modify" || time_val == "mtime";

        if !access_time && !mod_time {
            access_time = true;
            mod_time = true;
        }

        TouchFlags {
            access_time,
            mod_time,
            no_create: matches.is_present("nocreate") || matches.is_present("no_deref"),
            no_deref: matches.is_present("no_deref"),
            reference_path: matches.value_of("reference"),
            date: matches.value_of("date"),
            timestamp: matches.value_of("timestamp"),
        }
    }
}

/// Returns the correct `(atime, mtime)` acording to the `flags`.
fn new_filetimes(flags: TouchFlags) -> Result<(FileTime, FileTime), String> {
    if let Some(flags_date) = flags.date {
        let date = match PrimitiveDateTime::parse(&flags_date, "%Y-%m-%d %H:%M:%S") {
            Ok(dt) => dt.assume_utc(),
            Err(err) => return Err(format!("Problem parsing date arguments: {}", err)),
        };
        let time = FileTime::from_unix_time(date.timestamp(), date.microsecond());

        Ok((time, time))
    } else if let Some(flags_reference) = flags.reference_path {
        let file_meta = match fs::metadata(flags_reference) {
            Ok(m) => m,
            Err(err) => {
                return Err(format!(
                    "Failed to get {} (OTHER_FILE) metadata: {}",
                    flags_reference, err
                ));
            },
        };

        Ok((
            FileTime::from_last_access_time(&file_meta),
            FileTime::from_last_modification_time(&file_meta),
        ))
    } else if let Some(flags_timestamp) = flags.timestamp {
        // PrimitiveDateTime::parse doesn't handle missing %C and %C%y in format,
        // thus, based on the length of the input, the input string will be
        // completed up to the "%C%y%m%d%H%M.%S" format
        let current_date: PrimitiveDateTime = SystemTime::now().into();
        let input = flags_timestamp.trim_start();
        let input = match input.len() {
            // CCYYMMDDhhmm.ss format, nothing to add
            15 => input.to_owned(),
            // YYMMDDhhmm.ss format, add century
            13 => format!("{}{}", current_date.format("%C"), input),
            // CCYYMMDDhhmm format, add .00 for seconds
            12 => format!("{}.00", input),
            // MMDDhhmm.ss format, add year
            11 => format!("{}{}", current_date.format("%Y"), input),
            // YYMMDDhhmm format, add century and .00 for seconds
            10 => format!("{}{}.00", current_date.format("%C"), input),
            // YYMMDDhhmm format, add year and .00 for seconds
            8 => format!("{}{}.00", current_date.format("%Y"), input),
            _ => return Err(format!("Unhandled timestamp format '{}'", input)),
        };
        let date = match PrimitiveDateTime::parse(&input, "%Y%m%d%H%M.%S") {
            Ok(dt) => dt.assume_utc(),
            Err(err) => return Err(format!("Problem parsing timestamp argument: {}", err)),
        };
        let time = FileTime::from_unix_time(date.timestamp(), date.microsecond());

        Ok((time, time))
    } else {
        let now = FileTime::from_system_time(SystemTime::now());

        Ok((now, now))
    }
}

/// Update the times of the `path` acording with the `flags`.
fn update_time(
    path: &str, new_atime: FileTime, new_mtime: FileTime, meta: &Metadata, flags: TouchFlags,
) {
    match (flags.access_time, flags.mod_time) {
        (true, false) => update_access_time(&path, new_atime, meta, flags.no_deref),
        (false, true) => update_modification_time(&path, new_mtime, meta, flags.no_deref),
        (true, true) => update_both_time(&path, new_atime, new_mtime, flags.no_deref),

        // Unreachable because when creating `TouchFlags` if both are false, we change both to true
        // since de default behaviour is to change both. So (false, false) will never happen, and if
        // happen, it's a bug.
        _ => unreachable!(),
    }
}

fn update_access_time(path: &str, new_atime: FileTime, meta: &Metadata, no_deref: bool) {
    if no_deref {
        let mtime = FileTime::from_last_modification_time(meta);

        if let Err(err) = set_symlink_file_times(&path, new_atime, mtime) {
            eprintln!("touch: Failed to update {} access time: {}", &path, err);
        }
    } else if let Err(err) = set_file_atime(&path, new_atime) {
        eprintln!("touch: Failed to update {} access time: {}", &path, err);
    }
}

fn update_modification_time(path: &str, new_mtime: FileTime, meta: &Metadata, no_deref: bool) {
    if no_deref {
        let atime = FileTime::from_last_access_time(meta);

        if let Err(err) = set_symlink_file_times(&path, atime, new_mtime) {
            eprintln!("touch: Failed to update {} modification time: {}", &path, err);
        }
    } else if let Err(err) = set_file_mtime(&path, new_mtime) {
        eprintln!("touch: Failed to update {} modification time: {}", &path, err);
    }
}

fn update_both_time(path: &str, new_atime: FileTime, new_mtime: FileTime, no_deref: bool) {
    if no_deref {
        if let Err(err) = set_symlink_file_times(&path, new_atime, new_mtime) {
            eprintln!("touch: Failed to update {} time: {}", &path, err);
        }
    } else if let Err(err) = set_file_times(&path, new_atime, new_mtime) {
        eprintln!("touch: Failed to update {} time: {}", &path, err);
    }
}
