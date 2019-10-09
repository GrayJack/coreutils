#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::{ffi::CString, process};

use coreutils_core::utmpx::{UtmpxSet, UtmpxType};

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("users.yml");
    let _matches = App::from_yaml(yaml).get_matches();

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let uts = if _matches.is_present("FILE") {
        let file = _matches.value_of("FILE").unwrap();
        let file = match CString::new(file) {
            Ok(s) => s,
            Err(err) => {
                eprintln!("users: {}", err);
                process::exit(1);
            },
        };
        match UtmpxSet::from_file(&file) {
            Ok(u) => u,
            Err(_) => UtmpxSet::system(),
        }
    } else {
        UtmpxSet::system()
    };

    // When File is not possible, just ignore it
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let uts = UtmpxSet::system();

    if !uts.is_empty() {
        uts.iter()
            .filter(|u| u.utype() == UtmpxType::UserProcess)
            .for_each(|u| print!("{} ", u.user()));

        println!();
    }
}
