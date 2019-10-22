use std::{
    env::current_dir,
    fs::{self, DirEntry},
    io, process,
};

use clap::{load_yaml, App, ArgMatches};

#[derive(Debug, Clone, Copy)]
struct LsFlags {
    pub all: bool,
}

impl LsFlags {
    pub fn from_matches(matches: &ArgMatches) -> Self {
        let flags = LsFlags { all: matches.is_present("all") };

        return flags;
    }
}

fn main() {
    let yaml = load_yaml!("ls.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let flags = LsFlags::from_matches(&matches);

    let cwd = match current_dir() {
        Ok(path) => path.into_os_string().into_string().unwrap(),
        Err(err) => {
            eprintln!("ls: error reading current working directory: {}", err);
            process::exit(1);
        },
    };

    let mut dirs: Vec<String> = match matches.values_of("FILE") {
        Some(dirs) => dirs.map(String::from).collect(),
        None => vec![],
    };

    if dirs.is_empty() {
        dirs.push(cwd);
    }


    match ls(dirs, flags) {
        Ok(()) => {},
        Err(msg) => {
            eprintln!("ls: {}", msg);
            process::exit(1);
        },
    };
}

fn ls(dirs: Vec<String>, flags: LsFlags) -> io::Result<()> {
    for (_, dir) in dirs.iter().enumerate() {
        let paths = match fs::read_dir(dir) {
            Ok(paths) => paths,
            Err(err) => {
                eprintln!("ls: couldn't read {}: {}", dir, err);
                process::exit(1);
            },
        };

        let mut files: Vec<DirEntry> = paths.map(|f| f.unwrap()).collect();
        files.sort_by_key(|dir| dir.path());

        for entry in files {
            // let metadata = entry.metadata()?;
            // let permissions = metadata.permissions();
            let name = entry.file_name().into_string().unwrap();

            if name.starts_with(".") && !flags.all {
                continue;
            }

            println!("{}", name);
        }
    }
    Ok(())
}
