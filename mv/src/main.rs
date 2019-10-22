use clap::{load_yaml, App, AppSettings::ColoredHelp, ArgMatches};
use coreutils_core::{backup::*, input::*};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
enum OverwriteMode {
    Force,
    Interactive,
    NoClobber,
}

#[derive(Debug, Clone)]
struct MvFlags {
    backup: BackupMode,
    overwrite: OverwriteMode,
    update: bool,
    strip_trailing_slashes: bool,
    verbose: bool,
    suffix: String,
    target_directory: String,
    no_target_directory: bool,
}

impl OverwriteMode {
    pub fn from_matches(matches: &ArgMatches) -> OverwriteMode {
        let mut res = (OverwriteMode::Force, 0);

        if matches.is_present("force") && matches.index_of("force").unwrap() > res.1 {
            res = (OverwriteMode::Force, matches.index_of("force").unwrap());
        }

        if matches.is_present("interactive") && matches.index_of("interactive").unwrap() > res.1 {
            res = (OverwriteMode::Interactive, matches.index_of("interactive").unwrap());
        }

        if matches.is_present("noClobber") && matches.index_of("noClobber").unwrap() > res.1 {
            res = (OverwriteMode::NoClobber, matches.index_of("noClobber").unwrap());
        }

        res.0
    }
}

impl MvFlags {
    pub fn from_matches(matches: &ArgMatches) -> MvFlags {
        let target_dir = {
            if !matches.is_present("targetDirectory") {
                String::from("")
            } else {
                matches.value_of("targetDirectory").unwrap().to_string()
            }
        };

        MvFlags {
            backup: BackupMode::from_string(matches.value_of("backup").unwrap()),
            overwrite: OverwriteMode::from_matches(&matches),
            update: matches.is_present("update"),
            strip_trailing_slashes: matches.is_present("stripTrailingSlashes"),
            verbose: matches.is_present("verbose"),
            suffix: matches.value_of("suffix").unwrap().to_string(),
            target_directory: target_dir,
            no_target_directory: matches.is_present("noTargetDirectory"),
        }
    }
}

fn main() {
    let yaml = load_yaml!("mv.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();
    let flags = MvFlags::from_matches(&matches);

    let sources: Vec<PathBuf> = {
        let strip = flags.strip_trailing_slashes;

        matches
            .values_of("SOURCE")
            .unwrap()
            .map(Path::new)
            .map(|val| if strip { val.components().as_path() } else { val })
            .map(|val| val.to_owned())
            .collect()
    };

    let success = if flags.target_directory != "" {
        move_files(sources, PathBuf::from(&flags.target_directory), &flags)
    } else if !flags.no_target_directory && sources.last().unwrap().is_dir() {
        let target = sources.last().unwrap();
        move_files(sources[..sources.len() - 1].to_vec(), target.to_path_buf(), &flags)
    } else if sources.len() == 2 {
        rename_file(&sources[0], &sources[1], &flags)
    } else if sources.len() == 1 {
        eprintln!("mv: No target supplied");
        false
    } else {
        let target = sources.last().unwrap();
        move_files(sources[..sources.len() - 1].to_vec(), target.to_path_buf(), &flags)
    };

    if !success {
        std::process::exit(1);
    } else {
        std::process::exit(0);
    }
}

fn move_files(sources: Vec<PathBuf>, target: PathBuf, flags: &MvFlags) -> bool {
    if !target.is_dir() {
        eprintln!("mv: '{}' is not a directory", target.display());
        return false;
    }

    let mut success = true;
    for source in sources {
        match source.file_name() {
            Some(filename) => {
                let new = target.join(filename);

                if !rename_file(&source, &new, flags) {
                    success = false;
                }
            },
            None => {
                success = false;
                eprintln!("mv: Cannot 'stat' file '{}'", source.display());
            },
        }
    }

    return success;
}

fn rename_file(curr: &PathBuf, new: &PathBuf, flags: &MvFlags) -> bool {
    if new.exists() {
        match &flags.overwrite {
            OverwriteMode::Force => {},
            OverwriteMode::Interactive => {
                let input = Input::new()
                    .with_msg(&format!("mv: overwrite '{}'?", new.display()))
                    .with_err_msg("mv: could not read user input");
                if !input.is_affirmative() {
                    return true;
                }
            },
            OverwriteMode::NoClobber => return true,
        };

        if flags.update && file_older(curr, new) {
            return true;
        }

        let res = match &flags.backup {
            BackupMode::Numbered => Some(create_numbered_backup(new)),
            BackupMode::Existing => Some(create_existing_backup(new, &flags.suffix)),
            BackupMode::Simple => Some(create_simple_backup(new, &flags.suffix)),
            BackupMode::None => None,
        };

        if let Some(res) = res {
            match res {
                Ok(file) => println!("mv: Created backup file {}", file.display()),
                Err(err) => {
                    eprintln!("mv: Backup failed: {}", err);
                    return false;
                },
            };
        }
    }

    match fs::rename(curr, new) {
        Ok(()) => {
            if flags.verbose {
                println!("mv: Renamed {} to {}", curr.display(), new.display());
            }

            true
        },
        Err(msg) => {
            eprintln!("mv: Cannot rename {}: {}", curr.display(), msg);
            false
        },
    }
}

fn file_older(f: &PathBuf, ff: &PathBuf) -> bool {
    let f_attrs = match fs::metadata(f) {
        Ok(attrs) => attrs,
        Err(msg) => {
            eprintln!("mv: stat failed: {}", msg);
            return true;
        },
    };
    let ff_attrs = match fs::metadata(ff) {
        Ok(attrs) => attrs,
        Err(msg) => {
            eprintln!("mv: stat failed: {}", msg);
            return true;
        },
    };

    f_attrs.modified().unwrap() < ff_attrs.modified().unwrap()
}
