use std::{
    fs::{metadata, File},
    process,
    time::SystemTime,
};

use chrono::NaiveDateTime;
use clap::{load_yaml, App, AppSettings::ColoredHelp, ArgMatches};
use filetime::{set_file_atime, set_file_mtime, FileTime};

// TODO: add Unit tests for touch
#[cfg(test)]
mod tests;

fn main() {
    let yaml = load_yaml!("touch.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let flags = TouchFlags::from_matches(&matches);

    // get files list in argument
    // Required argument, ok to unwrap and not check if is supplied.
    let files: Vec<_> = matches.values_of("FILE").unwrap().collect();

    touch(&files, flags);
}

#[derive(Debug, Clone, Copy)]
struct TouchFlags<'a> {
    access_time: bool,
    no_create: bool,
    mod_time: bool,
    time: bool,
    time_val: &'a str,
    date: bool,
    date_val: &'a str,
}

impl<'a> TouchFlags<'a> {
    fn from_matches(matches: &'a ArgMatches<'a>) -> Self {
        TouchFlags {
            access_time: matches.is_present("accesstime"),
            no_create: matches.is_present("nocreate"),
            mod_time: matches.is_present("modification"),
            time: matches.is_present("time"),
            time_val: matches.value_of("time").unwrap_or(""),
            date: matches.is_present("date"),
            date_val: matches.value_of("date").unwrap_or(""),
        }
    }
}

fn touch(files: &[&str], flags: TouchFlags) {
    for filename in files {
        // if file already exist in the current directory
        let file_metadata = metadata(&filename);
        if file_metadata.is_err() && flags.no_create {
            match File::create(&filename) {
                Ok(_) => (),
                Err(e) => eprintln!("touch: Failed to create file {}: {}", &filename, e),
            }
        } else if flags.date {
            let native_date = NaiveDateTime::parse_from_str(&flags.date_val, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_else(|err| {
                    // If there is problems parsing the
                    eprintln!("touch: Problem parsing date arguments: {}", err);
                    process::exit(1);
                });
            let newfile_time = FileTime::from_unix_time(
                native_date.timestamp(),
                native_date.timestamp_subsec_millis(),
            );

            update_time(&filename, newfile_time, flags);
        } else {
            let newfile_time = FileTime::from_system_time(SystemTime::now());

            update_time(&filename, newfile_time, flags);
        }
    }
}

fn update_time(path: &str, filetime: FileTime, flags: TouchFlags) {
    if flags.access_time
        || flags.time_val == "access"
        || flags.time_val == "atime"
        || flags.time_val == "use"
    {
        update_access_time(&path, filetime);
    } else if flags.mod_time || flags.time_val == "modify" || flags.time_val == "mtime" {
        update_modification_time(&path, filetime);
    } else {
        update_access_time(&path, filetime);
        update_modification_time(&path, filetime);
    }
}

fn update_access_time(path: &str, filetime: FileTime) {
    match set_file_atime(&path, filetime) {
        Ok(_) => (),
        Err(e) => eprintln!("touch: Failed to update {} access time: {}", &path, e),
    };
}

fn update_modification_time(path: &str, filetime: FileTime) {
    match set_file_mtime(&path, filetime) {
        Ok(_) => (),
        Err(e) => eprintln!("touch: Failed to update {} modification time: {}", &path, e),
    };
}
