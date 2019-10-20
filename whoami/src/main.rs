use std::{env, process};

use coreutils_core::passwd::Passwd;

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("whoami.yml");
    let _matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let user = match Passwd::effective() {
        Ok(pw) => pw,
        Err(_) => {
            eprintln!("whoami: Failed to get user");
            process::exit(1);
        },
    };

    // If user name in Passwd is empty, check for environment variable USER.
    let usr_name = if user.name().is_empty() {
        if let Ok(name) = env::var("USER") {
            name
        } else {
            eprintln!("whoami: User name not found.");
            drop(user);
            process::exit(1);
        }
    } else {
        user.name().to_string()
    };

    println!("{}", usr_name);
}
