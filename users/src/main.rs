#[cfg(any(target_os = "openbsd"))]
use std::process;

#[cfg(not(any(target_os = "openbsd")))]
use coreutils_core::os::utmpx::{UtmpxSet as UtmpSet, UtmpxType};
#[cfg(any(target_os = "openbsd"))]
use coreutils_core::os::{utmp::UtmpSet, ByteSlice};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("users.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let uts = if matches.is_present("FILE") {
        let file = matches.value_of("FILE").unwrap();

        match UtmpSet::from_file(&file) {
            Ok(u) => u,
            #[cfg(not(any(target_os = "openbsd")))]
            Err(_) => UtmpSet::system(),
            #[cfg(any(target_os = "openbsd"))]
            Err(_) => match UtmpSet::system() {
                Ok(uu) => uu,
                Err(err) => {
                    eprintln!("users: failed to get utmp: {}", err);
                    process::exit(1);
                },
            },
        }
    } else {
        #[cfg(any(target_os = "openbsd"))]
        match UtmpSet::system() {
            Ok(u) => u,
            Err(err) => {
                eprintln!("users: failed to get utmp: {}", err);
                process::exit(1);
            },
        }

        #[cfg(not(any(target_os = "openbsd")))]
        UtmpSet::system()
    };

    if !uts.is_empty() {
        #[cfg(any(target_os = "openbsd"))]
        uts.iter()
            .filter(|u| match u.user().to_str() {
                Ok("") => false,
                Ok("shutdown") => false,
                Ok("reboot") => false,
                Ok(_) => true,
                Err(_) => false,
            })
            .for_each(|u| print!("{} ", u.user()));

        #[cfg(not(any(target_os = "openbsd")))]
        uts.iter()
            .filter(|u| u.entry_type() == UtmpxType::UserProcess)
            .for_each(|u| print!("{} ", u.user()));

        println!();
    }
}
