#[cfg(any(target_os = "openbsd"))]
use std::process;

#[cfg(not(any(target_os = "openbsd")))]
use coreutils_core::os::utmpx::{UtmpxKind, UtmpxSet as UtmpSet};
#[cfg(any(target_os = "openbsd"))]
use coreutils_core::{os::utmp::UtmpSet, ByteSlice};

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let uts = if let Some(file) = matches.value_of("FILE") {
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
            .filter(|u| u.entry_type() == UtmpxKind::UserProcess)
            .for_each(|u| print!("{} ", u.user()));

        println!();
    }
}
