#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::{ffi::CString, process};

use coreutils_core::utmpx::{UtmpxSet, UtmpxType};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("users.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let uts = if matches.is_present("FILE") {
        let file = matches.value_of("FILE").unwrap();

        match UtmpxSet::from_file(&file) {
            Ok(u) => u,
            Err(_) => UtmpxSet::system(),
        }
    } else {
        UtmpxSet::system()
    };

    if !uts.is_empty() {
        uts.iter()
            .filter(|u| u.utype() == UtmpxType::UserProcess)
            .for_each(|u| print!("{} ", u.user()));

        println!();
    }
}
