use std::{io, path::Path};

use coreutils_core::{
    group::{Group, Groups},
    passwd::Passwd,
    process::{set_group, set_groups, set_user, change_root},
};

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

    change_root(root).map_err(|e| eprintln!("error changing root {:?}", e)).unwrap();

    if let Some(groups_list) = matches.value_of("groups") {
        set_groups_from_list(groups_list)
            .expect("unable to set groups for process from group list");
    }

    if let Some(userspec_str) = matches.value_of("userspec") {
        set_user_from_userspec(userspec_str)
            .expect("unable to set groups for process from userspec");
    }

    if let Some(group) = matches.value_of("group") {
        let group = Group::from_name(group).expect("unable to get group information");
        set_group(group).expect("unable to set group for process");
    }

    if let Some(user) = matches.value_of("user") {
        let user = Passwd::from_name(user).expect("unable to get user from passwd");
        set_user(user).expect("unable to set user for process");
    }

    let proc = std::process::Command::new(cmd).args(args).status().unwrap();

    let exit_code = if proc.success() { 0 } else { proc.code().unwrap_or(1) };

    std::process::exit(exit_code);
}

fn set_groups_from_list(groups_list: &str) -> io::Result<()> {
    let groups: Vec<&str> = groups_list.split(",").collect();
    let groups = Groups::from_group_list(&groups).expect("unable to get groups from group list");
    set_groups(groups)
}

fn set_user_from_userspec(userspec: &str) -> io::Result<()> {
    let parts: Vec<&str> = userspec.split(':').collect();
    if parts.len() != 2 {
        eprintln!("userspec is in an incorrect format");
        std::process::exit(1);
    }

    let (user, group) = (parts[0], parts[1]);
    let user = Passwd::from_name(user).expect("unable to get user from name");
    let group = Group::from_name(group).expect("unable to get group from name");

    set_group(group)?;
    set_user(user)?;

    Ok(())
}
