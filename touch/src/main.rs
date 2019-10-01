use clap::{load_yaml, App, ArgMatches};
use filetime::{set_file_atime, set_file_mtime, FileTime};
use std::fs::{metadata, File};
use std::io::Result;
use std::process;
use std::time::SystemTime;

fn main() {
    let yaml = load_yaml!("touch.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let files = if matches.is_present("FILE") {
        matches.values_of("FILE").unwrap().collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    if files.len() > 0 {
        match touch(&matches, files) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("touch: Failed to write to stdout.\n{}", e);
                process::exit(1);
            }
        };
    } else {
        eprintln!("touch: a file name is required.\n");
        process::exit(1);
    }
}

fn touch(matches: &ArgMatches, files: Vec<&str>) -> Result<()> {
    for filename in files {
        let file_metadata = metadata(&filename);
        let no_create = !matches.is_present("nocreate");
        if file_metadata.is_err() && no_create {
            match File::create(&filename) {
                Ok(_) => (),
                Err(e) => eprintln!("touch: Failed to create file {} : {}", filename, e),
            }
        } else {
            let newfile_time = FileTime::from_system_time(SystemTime::now());
            update_time(matches, &filename, newfile_time);
        }
    }
    Ok(())
}

fn update_time(matches: &ArgMatches, path: &str, filetime: FileTime) {
    let access_time = matches.is_present("accesstime");
    let modification = matches.is_present("modification");
    let time = matches.value_of("time").unwrap();

    if access_time || time == "atime" {
        update_access_time(&path, filetime);
    } else if modification || time == "mtime" {
        update_modification_time(&path, filetime);
    } else {
        update_access_time(&path, filetime);
        update_modification_time(&path, filetime);
    }
}

fn update_access_time(path: &str, filetime: FileTime) {
    match set_file_atime(&path, filetime) {
        Ok(_) => (),
        Err(e) => eprintln!("touch: Failed to update {} access time \n {}", &path, e),
    };
}

fn update_modification_time(path: &str, filetime: FileTime) {
    match set_file_mtime(&path, filetime) {
        Ok(_) => (),
        Err(e) => eprintln!(
            "touch: Failed to update {} modification time \n {}",
            &path, e
        ),
    };
}

//TODO: add Unit tests for touch
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_create_empty_files() {
        let matches = ArgMatches::new();
        let files = vec!["file1.js", "file2.js"];

        match touch(&matches, files) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("touch: Failed to write to stdout.\n{}", e);
                process::exit(1);
            }
        };
    }
}
