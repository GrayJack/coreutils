use chrono::NaiveDateTime;
use clap::{load_yaml, App, ArgMatches};
use filetime::{set_file_atime, set_file_mtime, FileTime};
use std::{
    fs::{metadata, File},
    io::Result,
    process,
    time::SystemTime,
};

fn main() {
    let yaml = load_yaml!("touch.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // get files list in argument
    let files = if matches.is_present("FILE") {
        matches.values_of("FILE").unwrap().collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    if !files.is_empty() {
        touch(&matches, &files).unwrap();
    } else {
        eprintln!("touch: a file name is required.\n");
        process::exit(1);
    }
}

fn touch(matches: &ArgMatches, files: &Vec<&str>) -> Result<()> {
    for filename in files {
        // if file already exist in the current directory
        let file_metadata = metadata(&filename);
        let no_create = !matches.is_present("nocreate");
        if file_metadata.is_err() && no_create {
            match File::create(&filename) {
                Ok(_) => (),
                Err(e) => eprintln!("touch: Failed to create file {} : {}", &filename, e),
            }
        } else {
            let date = matches.is_present("date");
            if date {
                let date_value = matches.value_of("date").unwrap();
                let native_date = NaiveDateTime::parse_from_str(&date_value, "%Y-%m-%d %H:%M:%S")
                    .unwrap_or_else(|err| {
                        eprintln!("Problem parsing date arguments: {}", err);
                        process::exit(1);
                    });
                let newfile_time = FileTime::from_unix_time(
                    native_date.timestamp(),
                    native_date.timestamp_subsec_millis(),
                );
                update_time(matches, &filename, newfile_time);
            } else {
                let newfile_time = FileTime::from_system_time(SystemTime::now());
                update_time(matches, &filename, newfile_time);
            }
        }
    }
    Ok(())
}

fn update_time(matches: &ArgMatches, path: &str, filetime: FileTime) {
    let access_time = matches.is_present("accesstime");
    let modification = matches.is_present("modification");
    let time = matches.is_present("time");
    let time_value = if time {
        matches.value_of("time").unwrap()
    } else {
        ""
    };

    if access_time || time_value == "atime" {
        update_access_time(&path, filetime);
    } else if modification || time_value == "mtime" {
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
    use std::fs::remove_file;

    #[test]
    fn touch_create_empty_files() {
        let matches = ArgMatches::new();
        let files = vec!["file1.rs", "file2.rs"];

        match touch(&matches, &files) {
            Ok(_) => (),
            Err(e) => eprintln!("touch: Failed to create file {}", e),
        };

        assert_eq!(metadata("file1.rs").is_ok(), true);
        assert_eq!(metadata("file2.rs").is_ok(), true);
        remove_test_files(&files).unwrap();
    }

    #[test]
    fn touch_update_existing_files() {
        let matches = ArgMatches::new();
        let files = vec!["file3.rs", "file4.rs"];

        File::create("file3.rs").unwrap();
        let file1_metadata = metadata("file3.rs").unwrap();
        let file1_mtime = FileTime::from_last_modification_time(&file1_metadata);
        let file1_atime = FileTime::from_last_access_time(&file1_metadata);

        //update and create files
        match touch(&matches, &files) {
            Ok(_) => (),
            Err(e) => eprintln!("touch: Failed to create file {}", e),
        };

        let new_file1_metadata = metadata("file3.rs").unwrap();
        let new_file1_mtime = FileTime::from_last_modification_time(&new_file1_metadata);
        let new_file1_atime = FileTime::from_last_access_time(&new_file1_metadata);
        // check that file1 modification time has changed
        assert_ne!(file1_mtime, new_file1_mtime);

        //check that file1 access time has changed
        assert_ne!(file1_atime, new_file1_atime);

        remove_test_files(&files).unwrap();
    }

    #[test]
    fn touch_update_only_access_time() {
        let yaml = load_yaml!("touch.yml");
        let matches =
            App::from_yaml(yaml).get_matches_from(vec!["touch", "-a", "file5.rs", "file6.rs"]);

        let files = if matches.is_present("FILE") {
            matches.values_of("FILE").unwrap().collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        File::create(&files[0]).unwrap();

        let mut file1_metadata = metadata(&files[0]).unwrap();
        let file1_atime = FileTime::from_last_access_time(&file1_metadata);

        //update and create files
        match touch(&matches, &files) {
            Ok(_) => (),
            Err(e) => eprintln!("touch: Failed to create file {}", e),
        };

        file1_metadata = metadata(&files[0]).unwrap();
        let new_file1_atime = FileTime::from_last_access_time(&file1_metadata);

        //check that first file access time has changed
        assert_ne!(file1_atime, new_file1_atime);
        remove_test_files(&files).unwrap();
    }

    #[test]
    fn touch_update_only_modification_time() {
        let yaml = load_yaml!("touch.yml");
        let matches =
            App::from_yaml(yaml).get_matches_from(vec!["touch", "-m", "file7.rs", "file8.rs"]);

        let files = if matches.is_present("FILE") {
            matches.values_of("FILE").unwrap().collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        File::create(&files[0]).unwrap();

        let mut file1_metadata = metadata(&files[0]).unwrap();
        let file1_mtime = FileTime::from_last_modification_time(&file1_metadata);

        //update and create files
        touch(&matches, &files).unwrap();

        file1_metadata = metadata(&files[0]).unwrap();
        let new_file1_mtime = FileTime::from_last_modification_time(&file1_metadata);

        // check that first file modification time has changed
        assert_ne!(file1_mtime, new_file1_mtime);

        remove_test_files(&files).unwrap();
    }

    #[test]
    fn touch_update_time_with_date() {
        let yaml = load_yaml!("touch.yml");
        let matches = App::from_yaml(yaml).get_matches_from(vec![
            "touch",
            "-d=2009-01-03 03:13:00",
            "file9.rs",
            "file10.rs",
        ]);

        let files = if matches.is_present("FILE") {
            matches.values_of("FILE").unwrap().collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        File::create(&files[0]).unwrap();

        //update and create files
        touch(&matches, &files).unwrap();

        let file1_metadata = metadata(&files[0]).unwrap();
        let file1_mtime = FileTime::from_last_modification_time(&file1_metadata);

        let time =
            NaiveDateTime::from_timestamp(file1_mtime.unix_seconds(), file1_mtime.nanoseconds());

        // check modification and access time is equal 2009-01-03 03:13:00
        assert_eq!(
            time,
            NaiveDateTime::parse_from_str("2009-01-03 03:13:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        remove_test_files(&files).unwrap();
    }
    fn remove_test_files(files: &Vec<&str>) -> Result<()> {
        for filename in files {
            remove_file(&filename)?;
        }
        Ok(())
    }
}
