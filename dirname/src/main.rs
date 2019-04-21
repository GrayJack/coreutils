use std::path::Path;

use clap::{App, load_yaml};

fn main() {
    let yaml = load_yaml!("dirname.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let line_ending = if matches.is_present("zero") { '\0' } else { '\n' };

    let paths = matches.values_of("NAME").unwrap();
    for path in paths {
        print!("{} {}", basedir(path), line_ending)
    }
}

fn basedir(path: &str) -> String {
    let p = Path::new(path);
    match p.parent() {
        Some(dir) => {
            if dir.components().next() == None {
                ".".to_string()
            } else {
                dir.to_string_lossy().to_string()
            }
        },
        None => {
            if p.is_absolute() || path == "/" {
                "/".to_string()
            } else {
                ".".to_string()
            }
        }
    }
}
