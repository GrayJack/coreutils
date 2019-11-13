use std::{env, process};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("logname.yml");
    let _matches = App::from_yaml(yaml)
        .settings(&[ColoredHelp])
        .help_message("Display help information")
        .version_message("Display version information")
        .get_matches();

    let login_name = if let Ok(name) = env::var("LOGNAME") {
        name
    } else {
        eprintln!("logname: No login name found.");
        process::exit(2);
    };

    println!("{}", login_name);
}
