use std::process;

use coreutils_core::{group::Group, passwd::Passwd};

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("id.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let audit_flag = matches.is_present("audit");
    let group_flag = matches.is_present("group");
    let groups_flag = matches.is_present("groups");
    let user_flag = matches.is_present("user");
    let name_flag = matches.is_present("name");
    let zero_flag = matches.is_present("zero");
    let file_flag = matches.is_present("file");
    let real_flag = matches.is_present("real");
    let pretty_flag = matches.is_present("pretty") || matches.is_present("human");
    let by_name = matches.is_present("USER");

    let mut sep = '\n';

    if audit_flag && cfg!(target_os = "freebsd") {
        audit_logic();
    }

    // Checks if zero_flag is being used as expected
    if zero_flag {
        if let (false, false, false, false) = (group_flag, groups_flag, user_flag, file_flag) {
            eprintln!("id: Option --zero not permitted in pretty or default format");
            process::exit(1);
        } else {
            sep = '\0'
        }
    }

    // Checks if name_flag is being used as expected
    if name_flag {
        // If `--name` doesn't occour with `--group` or `groups` or `user`, it errors out
        if let (false, false, false) = (group_flag, groups_flag, user_flag) {
            eprintln!("id: Cannot print only names or real IDs in default format");
            process::exit(1);
        }
    }

    // Checks if real_flag is being used as expected
    if real_flag {
        if let (false, false) = (group_flag, user_flag) {
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
    } else if (user_flag || group_flag) && real_flag {
        Passwd::real()
    } else {
        Passwd::effective()
    };

    let passwd = match passwd {
        Ok(pw) => pw,
        Err(err) => {
            eprintln!("id: {}", err);
            process::exit(1);
        }
    };

    if user_flag {
        user_logic(&passwd, name_flag, sep);
    }

    if group_flag {
        group_logic(&passwd, name_flag, sep);
    }

    if groups_flag {
        groups_logic(&passwd, name_flag, sep);
    }

    if pretty_flag {
        pretty_logic(&passwd, sep);
    }

    if file_flag {
        print!("{}{}", passwd, sep);
        process::exit(0);
    }

    default_logic(&passwd, sep);
}

fn default_logic(passwd: &Passwd, sep: char) {
    let groups = match passwd.belongs_to() {
        Ok(gs) => gs,
        Err(err) => {
            eprintln!("id: {}", err);
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
    let final_pos = groups.len() - 1;
    for (i, group) in groups.into_iter().enumerate() {
        if i == final_pos {
            print!("{}({})", group.id(), group.name());
        } else {
            print!("{}({}),", group.id(), group.name());
        }
    }
    print!("{}", sep);
}

fn group_logic(passwd: &Passwd, name_flag: bool, sep: char) {
    if name_flag {
        let group = match Group::from_gid(passwd.gid()) {
            Ok(g) => g,
            Err(err) => {
                eprintln!("id: {}", err);
                process::exit(1);
            }
        };
        print!("{}{}", group.name(), sep);
        process::exit(0);
    }
    print!("{}{}", passwd.gid(), sep);
    process::exit(0);
}

fn user_logic(passwd: &Passwd, name_flag: bool, sep: char) {
    if name_flag {
        print!("{}{}", passwd.name(), sep);
        process::exit(0);
    }
    print!("{}{}", passwd.uid(), sep);
    process::exit(0);
}

fn groups_logic(passwd: &Passwd, name_flag: bool, sep: char) {
    let groups = match passwd.belongs_to() {
        Ok(gs) => gs,
        Err(err) => {
            eprintln!("id: {}", err);
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

fn pretty_logic(passwd: &Passwd, sep: char) {
    let groups = match passwd.belongs_to() {
        Ok(gs) => gs,
        Err(err) => {
            eprintln!("id: {}", err);
            process::exit(1);
        }
    };

    print!("uid\t\t{}{}groups\t", passwd.name(), sep);
    groups.into_iter().for_each(|g| print!("{} ", g.name()));
    print!("{}", sep);

    process::exit(0);
}

#[cfg(not(target_os = "freebsd"))]
fn audit_logic() {}

#[cfg(target_os = "freebsd")]
fn audit_logic() {
    match coreutils_core::audit::auditid() {
        Ok(_) => process::exit(0),
        Err(err) => {
            println!("id: {}", err);
            process::exit(1);
        }
    };
}
