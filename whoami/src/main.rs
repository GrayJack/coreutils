use std::{
    env,
    process,
};

use coreutils_core::passwd::Passwd;

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("whoami.yml");
    let _matches = App::from_yaml(yaml).get_matches();

    let user = Passwd::new();

    // If user name in Passwd is empty, check for environment variable USER.
    let usr_name = if user.name().is_empty() {
        if let Ok(name) = env::var("USER") {
            name
        } else {
            eprintln!("User name not found.");
            process::exit(2);
        }
    } else {
        user.name().to_owned()
    };

    println!("{}", usr_name);
}
