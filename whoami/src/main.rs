use std::{env, process};

use coreutils_core::os::passwd::Passwd;

mod cli;

fn main() {
    let _matches = cli::create_app().get_matches();

    let user = if let Ok(pw) = Passwd::effective() {
        pw
    } else {
        eprintln!("whoami: Failed to get user");
        process::exit(1);
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
