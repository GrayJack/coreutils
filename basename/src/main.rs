use clap::{App, load_yaml};

fn main() {
    let yaml = load_yaml!("basename.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let multiple_paths = matches.is_present("multiple") || matches.is_present("suffix");

    let suffix = if matches.is_present("suffix") {
        // Using unwrap here is ok since we already checked if --suffix/-s was used or not.
        matches.value_of("suffix").unwrap()
    } else {
        ""
    };

    let line_ending = if matches.is_present("zero") { '\0' } else { '\n' };

    if !multiple_paths {
        let path = matches.value_of("NAME").unwrap();
        print!("{} {}", basename(path, suffix), line_ending);
    } else {
        let paths = matches.values_of("NAME").unwrap();
        for path in paths {
            print!("Base {} {}", basename(path, suffix), line_ending);
        }
    }
}

fn basename(full_path: &str, suffix: &str) -> String {
    let split_full_path: Vec<&str> = full_path.split('/').collect();
    let path = match split_full_path.last() {
        Some(name) => strip_suffix(name, suffix).to_owned(),
        None => "".to_owned()
    };
    path.to_string()
}

fn strip_suffix(name: &str, suffix: &str) -> String {
    if name == suffix {
        return name.to_owned();
    }
    if name.ends_with(suffix) {
        return name[..(name.len() - suffix.len())].to_owned();
    }
    name.to_owned()
}
