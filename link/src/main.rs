use std::{fs, process};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("link.yml");
    let matches = App::from_yaml(yaml)
        .settings(&[ColoredHelp])
        .help_message("Display help information")
        .version_message("Display version information")
        .get_matches();

    let source = matches.value_of("FILE1").unwrap();
    let target = matches.value_of("FILE2").unwrap();

    if let Err(err) = fs::hard_link(source, target) {
        eprintln!("link: cannot create link '{}' to '{}': {}", target, source, err);
        process::exit(1);
    }
}
