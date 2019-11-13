use std::{fs, process};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("unlink.yml");
    let matches = App::from_yaml(yaml)
        .settings(&[ColoredHelp])
        .help_message("Display help information")
        .version_message("Display version information")
        .get_matches();

    let path = matches.value_of("FILE").unwrap();

    // Note: std::fs::remove_file corresponds to unlink(2) at time of this writing, but that
    // may change in the future according to the documentation.
    if let Err(err) = fs::remove_file(path) {
        eprintln!("unlink: cannot unlink '{}': {}", path, err);
        process::exit(1);
    }
}
