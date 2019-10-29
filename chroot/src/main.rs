use std::{convert::TryInto, ffi::CString, path::Path};

use coreutils_core::{group::Group, libc, passwd::Passwd};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("chroot.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let root = matches.value_of("NEWROOT").unwrap();
    let cmd = matches.value_of("COMMAND").unwrap_or("/bin/sh");
    let args: Vec<&str> = match matches.values_of("ARGS") {
        Some(args) => args.collect(),
        None => vec!["-i"],
    };

    change_root(Path::new(root)).map_err(|e| eprintln!("error changing root {:?}", e)).unwrap();

    if let Some(groups_list) = matches.value_of("groups") {
        set_groups_from_list(groups_list);
    }

    if let Some(userspec_str) = matches.value_of("userspec") {
        set_user_from_userspec(userspec_str);
    }

    if let Some(group) = matches.value_of("group") {
        let group = Group::from_name(group).expect("unable to get group information");
        set_group(group.id()).expect("unable to set group for process");
    }

    if let Some(user) = matches.value_of("user") {
        let user = Passwd::from_name(user).expect("unable to get user from passwd");
        set_user(user.uid()).expect("unable to set user for process");
    }

    let proc = std::process::Command::new(cmd).args(args).status().unwrap();

    let exit_code = if proc.success() { 0 } else { proc.code().unwrap_or(1) };

    std::process::exit(exit_code);
}

fn change_root(path: &Path) -> std::io::Result<()> {
    std::env::set_current_dir(path)?;

    let error = unsafe {
        libc::chroot(CString::new(".").unwrap().as_bytes_with_nul().as_ptr() as *const libc::c_char)
    };

    match error {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error()),
    }
}

fn set_groups_from_list(groups_list: &str) {
    let groups: Vec<&str> = groups_list.split(",").collect();
    let gids: Vec<libc::gid_t> = groups
        .into_iter()
        .map(|group| Group::from_name(group))
        .filter_map(Result::ok)
        .map(|group| group.id())
        .collect();
    set_groups(gids).expect("unable to set groups for process");
}

fn set_user_from_userspec(userspec: &str) {
    let parts: Vec<&str> = userspec.split(':').collect();
    if parts.len() != 2 {
        eprintln!("userspec is in an incorrect format");
        std::process::exit(1);
    }

    let (user, group) = (parts[0], parts[1]);
    let user = Passwd::from_name(user).expect("unable to get user from passwd");
    let group = Group::from_name(group).expect("unable to get group from name");

    set_group(group.id()).expect("unable to set group for process");
    set_user(user.uid()).expect("unable to set user for process");
}

fn set_user(user_id: libc::uid_t) -> std::io::Result<()> {
    match unsafe { libc::setuid(user_id) } {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error()),
    }
}

fn set_group(group_id: libc::gid_t) -> std::io::Result<()> {
    match unsafe { libc::setgid(group_id) } {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error()),
    }
}

fn set_groups(group_list: Vec<libc::gid_t>) -> std::io::Result<()> {
    match unsafe { libc::setgroups(group_list.len().try_into().unwrap(), group_list.as_ptr()) } {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error()),
    }
}
