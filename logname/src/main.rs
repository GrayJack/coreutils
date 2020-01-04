use std::{env, process};

use coreutils_core::login_name::user_login_name;

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("logname.yml");
    let _matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

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
