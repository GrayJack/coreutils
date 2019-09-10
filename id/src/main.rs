use std::process;

use coreutils_core::{group::Group, passwd::Passwd};

use clap::{load_yaml, App};

// TODO: Lacks audit flag and code
fn main() {
    let yaml = load_yaml!("id.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let group_flag = matches.is_present("group");
    let groups_flag = matches.is_present("groups");
    let user_flag = matches.is_present("user");
    let name_flag = matches.is_present("name");
    let zero_flag = matches.is_present("zero");
    let pretty_flag = matches.is_present("pretty") || matches.is_present("human");
    let by_name = matches.is_present("USER");

    let mut sep = '\n';

    if zero_flag {
        if let (false, false, false) = (group_flag, groups_flag, user_flag) {
            eprintln!("id: Option --zero not permitted in default format");
            process::exit(1);
        } else {
            sep = '\0'
        }
    }

    if name_flag {
        // If `--name` doesn't occour with `--group` or `groups` or `user`, it errors out
        if let (false, false, false) = (group_flag, groups_flag, user_flag) {
            eprintln!("id: Cannot print only names or real IDs in default format");
            process::exit(1);
        }
    }

    let name = if by_name {
        matches.value_of("USER").unwrap()
    } else {
        ""
    };

    let passwd = if by_name {
        Passwd::from_name(&name)
    } else {
        Passwd::new()
    };

    let passwd = match passwd {
        Ok(pw) => pw,
        Err(err) => {
            eprintln!("{:#?}", err);
            process::exit(1);
        }
    };

    if pretty_flag {
        let groups = match passwd.belongs_to() {
            Ok(gs) => gs,
            Err(err) => {
                eprintln!("{}", err);
                process::exit(1);
            }
        };

        print!("uid\t\t{}{}groups\t", passwd.name(), sep);
        groups.into_iter().for_each(|g| print!("{} ", g.name()));
        print!("{}", sep);

        process::exit(0);
    }

    if user_flag {
        if name_flag {
            print!("{}{}", passwd.name(), sep);
            process::exit(0);
        }
        print!("{}{}", passwd.uid(), sep);
        process::exit(0);
    }

    if group_flag {
        if name_flag {
            let group = match Group::from_gid(passwd.gid()) {
                Ok(g) => g,
                Err(err) => {
                    eprintln!("{}", err);
                    process::exit(1);
                }
            };
            print!("{}{}", group.name(), sep);
            process::exit(0);
        }
        print!("{}{}", passwd.gid(), sep);
        process::exit(0);
    }

    if groups_flag {
        let groups = match passwd.belongs_to() {
            Ok(gs) => gs,
            Err(err) => {
                eprintln!("{}", err);
                process::exit(1);
            }
        };
        if name_flag {
            groups.into_iter().for_each(|g| print!("{} ", g.name()));
            print!("{}", sep);
            process::exit(0);
        }
        groups.into_iter().for_each(|g| print!("{} ", g.id()));
        print!("{}", sep);
        process::exit(0);
    }

    let groups = match passwd.belongs_to() {
        Ok(gs) => gs,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
    .into_vec();

    print!(
        "uid={}({}) gid={}({}) groups=",
        passwd.uid(),
        passwd.name(),
        passwd.gid(),
        groups[0].name()
    );
    groups
        .iter()
        .for_each(|g| print!("{}({}),", g.id(), g.name()));
    print!("{}", sep);
}
