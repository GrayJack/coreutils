use std::{
    env,
    ffi::CStr,
    mem,
    os::raw::c_char,
    process,
    ptr::null_mut
};

use libc;

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("whoami.yml");
    let _matches = App::from_yaml(yaml).get_matches();

    // Got this size from manual page about getpwuid_r
    let mut buffer = [0; 16384];
    let usr_id = get_user_uid(&mut buffer);

    // Check by user id first, if not found, look for USER environment variable
    let usr_name = if let Some(name) = username(usr_id) {
        name
    } else if let Ok(name) = env::var("USER") {
        name
    } else {
        eprintln!("User name found.");
        process::exit(2);
    };

    println!("{}", usr_name);
}

fn username(usr_id: libc::passwd) -> Option<String> {
    let usr_name = usr_id.pw_name;

    if usr_name.is_null() {
        return None;
    }

    let usr_name = unsafe { CStr::from_ptr(usr_name).to_string_lossy().to_string() };

    Some(usr_name)
}

fn get_user_uid(buffer: &mut [c_char; 16384]) -> libc::passwd {
    let mut pwent: libc::passwd = unsafe { mem::zeroed() };
    let mut pwentp = null_mut();

    unsafe {
        libc::getpwuid_r(
            libc::geteuid(),
            &mut pwent,
            &mut buffer[0],
            buffer.len(),
            &mut pwentp,
        );
    }

    pwent
}
