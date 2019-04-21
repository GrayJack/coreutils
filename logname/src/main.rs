use std::{
    env,
    process
};

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("logname.yml");
    let _matches = App::from_yaml(yaml).get_matches();

    let login_name = if let Ok(name) = env::var("LOGNAME") {
        name
    } else {
        eprintln!("No login name found.");
        process::exit(2);
    };

    println!("{}", login_name);
}
