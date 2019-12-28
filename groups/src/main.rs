use coreutils_core::{
    group::{Error as GrError, Groups},
    passwd::Error as PwError,
};

use GrError::*;
use PwError::*;

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("groups.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let by_name = matches.is_present("USER");
    let id = matches.is_present("id");

    let name = matches.value_of("USER").unwrap_or("");

    let groups = if by_name {
        match Groups::from_username(name) {
            Ok(g) => g,
            Err(Passwd(box_err)) => match Box::leak(box_err) {
                PasswdNotFound => {
                    eprintln!("groups: Unknown user {}", name);
                    std::process::exit(1);
                },
                a => {
                    eprintln!("groups: {}", a);
                    std::process::exit(1);
                },
            },
            Err(err) => {
                eprintln!("groups: {}", err);
                std::process::exit(1);
            },
        }
    } else {
        match Groups::caller() {
            Ok(g) => g,
            Err(err) => {
                eprintln!("groups: {}", err);
                std::process::exit(1);
            },
        }
    };

    if !groups.is_empty() {
        if id {
            groups.iter().for_each(|g| print!("{}:{} ", g.name(), g.id()));
        } else {
            groups.iter().for_each(|g| print!("{} ", g.name()));
        }
    }
    println!();
}
