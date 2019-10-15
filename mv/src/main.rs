use self::backup::*;
use self::input::Input;
use clap::{load_yaml, App, ArgMatches};
use std::fs;
use std::path::{Path, PathBuf};

// TODO(gab): Extract this to core because cp, ln, etc use backups
pub mod backup {
    use regex::Regex;
    use std::fs;
    use std::io::{Error, ErrorKind};
    use std::path::PathBuf;

    #[derive(Debug, Clone, PartialEq)]
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

    pub fn create_numbered_backup(file: &PathBuf) -> Result<PathBuf, Error> {
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

    pub fn create_existing_backup(file: &PathBuf, suffix: &String) -> Result<PathBuf, Error> {
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

    pub fn create_simple_backup(file: &PathBuf, suffix: &String) -> Result<PathBuf, Error> {
        let new = PathBuf::from(format!("{}{}", file.display(), suffix));

        match fs::rename(file, &new) {
            Ok(()) => Ok(new),
            Err(error) => Err(error),
        }
    }
}

// TODO(gab): extract to core because a tonne of core utils use this
mod input {
    use std::io;
    use std::io::prelude::*;
    use std::process;

    #[derive(Debug)]
    pub struct Input(String);

    impl Input {
        pub fn new() -> Self {
            let mut line = String::new();
            match io::stdin().lock().read_line(&mut line) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("rm: cannot read input: {}", err);
                    process::exit(1);
                }
            };

            Input(line)
        }

        pub fn with_msg(msg: &str) -> Self {
            print!("{}", msg);

            if let Err(err) = io::stdout().lock().flush() {
                eprintln!("rm: could not flush stdout: {}", err);
                process::exit(1);
            }

            Self::new()
        }

        pub fn is_affirmative(&self) -> bool {
            let input = self.0.trim().to_uppercase();

            input == "Y" || input == "YES" || input == "1"
        }
    }
}

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
    let matches = App::from_yaml(yaml).get_matches();
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

    if flags.target_directory != "" {
        move_files(sources, PathBuf::from(&flags.target_directory), &flags);
    } else if !flags.no_target_directory && sources.last().unwrap().is_dir() {
        let target = sources.last().unwrap();
        move_files(sources[..sources.len() - 1].to_vec(), target.to_path_buf(), &flags);
    } else if sources.len() == 2 {
        rename_file(&sources[0], &sources[1], &flags);
    } else {
        // TODO(gab): make this better
        println!("Erroring out!");
        println!("Source: {:?}", sources);
        println!("Flags: {:?}", flags);
    }
}

fn move_files(sources: Vec<PathBuf>, target: PathBuf, flags: &MvFlags) {
    // TODO(gab): fill out function
    println!("move files to directory: {:?} -> {:?}", sources, target);
    println!("Flags: {:?}", flags);

    
}

fn rename_file(curr: &PathBuf, new: &PathBuf, flags: &MvFlags) {
    if new.exists() {
        match &flags.overwrite {
            OverwriteMode::Force => {},
            OverwriteMode::Interactive => {
                if !Input::with_msg(&format!("mv: overwrite '{}'?", new.display())).is_affirmative()
                {
                    return;
                }
            }
            OverwriteMode::NoClobber => return,
        };

        if flags.update && file_older(curr, new) {
            return;
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
                Err(err) => eprintln!("mv: Backup failed: {}", err),
            };
        }
    }

    match fs::rename(curr, new) {
        Ok(()) => {
            if flags.verbose {
                println!("mv: Renamed {} to {}", curr.display(), new.display());
            }
        }
        Err(msg) => eprintln!("mv: Cannot rename {}: {}", curr.display(), msg),
    };
}

fn file_older(f: &PathBuf, ff: &PathBuf) -> bool {
    let f_attrs = match fs::metadata(f) {
        Ok(attrs) => attrs,
        Err(msg) => {
            eprintln!("mv: stat failed: {}", msg);
            return true;
        }
    };
    let ff_attrs = match fs::metadata(ff) {
        Ok(attrs) => attrs,
        Err(msg) => {
            eprintln!("mv: stat failed: {}", msg);
            return true;
        }
    };

    f_attrs.modified().unwrap() < ff_attrs.modified().unwrap()
}
