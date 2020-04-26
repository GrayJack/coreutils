use std::{env, process};

use coreutils_core::os::login_name::user_login_name;

mod cli;

fn main() {
    let _matches = cli::create_app().get_matches();

    let login_name = if let Some(name) = user_login_name() {
        format!("{}", name)
    } else if let Ok(name) = env::var("LOGNAME") {
        name
    } else {
        eprintln!("logname: No login name found.");
        process::exit(2);
    };

    println!("{}", login_name);
}
