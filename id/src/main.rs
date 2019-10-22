use std::process;

use coreutils_core::{group::Group, passwd::Passwd};

use clap::{load_yaml, App, AppSettings::ColoredHelp, ArgMatches};

fn main() {
    let yaml = load_yaml!("id.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let flags = IdFlags::from_matches(&matches);

    let mut sep = '\n';

    if flags.audit && (cfg!(target_os = "freebsd") || cfg!(target_os = "macos")) {
        audit_logic();
        return;
    }

    if flags.rtable && cfg!(target_os = "openbsd") {
        rtable_logic();
        return;
    }

    // Checks if zero_flag is being used as expected
    if flags.zero {
        if !flags.is_zero_valid() {
            eprintln!("id: Option --zero not permitted in pretty or default format");
            process::exit(1);
        } else {
            sep = '\0'
        }
    }

    // Checks if name_flag is being used as expected
    if !flags.is_name_valid() {
        eprintln!("id: Cannot print only names or real IDs in default format");
        process::exit(1);
    }

    // Checks if real_flag is being used as expected
    if flags.is_real_valid() {
        eprintln!("id: Cannot print only names or real IDs in default format");
        process::exit(1);
    }

    let name = if flags.by_name { matches.value_of("USER").unwrap() } else { "" };

    let passwd = if flags.by_name {
        Passwd::from_name(&name)
    } else if (flags.user || flags.group) && flags.real {
        Passwd::real()
    } else {
        Passwd::effective()
    };

    let passwd = match passwd {
        Ok(pw) => pw,
        Err(err) => {
            eprintln!("id: {}", err);
            process::exit(1);
        },
    };

    if flags.user {
        user_logic(&passwd, flags, sep);
        return;
    }

    if flags.group {
        group_logic(&passwd, flags, sep);
        return;
    }

    if flags.groups {
        groups_logic(&passwd, flags, sep);
        return;
    }

    if flags.pretty {
        pretty_logic(&passwd, sep);
        return;
    }

    if flags.file {
        print!("{}{}", passwd, sep);
        return;
    }

    default_logic(&passwd, sep);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct IdFlags {
    audit:   bool,
    by_name: bool,
    file:    bool,
    group:   bool,
    groups:  bool,
    pretty:  bool,
    name:    bool,
    real:    bool,
    rtable:  bool,
    user:    bool,
    zero:    bool,
}

impl IdFlags {
    fn from_matches(matches: &ArgMatches<'_>) -> Self {
        IdFlags {
            audit:   matches.is_present("audit"),
            by_name: matches.is_present("USER"),
            file:    matches.is_present("file"),
            group:   matches.is_present("group"),
            groups:  matches.is_present("groups"),
            name:    matches.is_present("name"),
            pretty:  matches.is_present("pretty") | matches.is_present("human"),
            real:    matches.is_present("real"),
            rtable:  matches.is_present("rtable"),
            user:    matches.is_present("user"),
            zero:    matches.is_present("zero"),
        }
    }

    /// Check if `--zero` doesn't occour with `--group` or `--groups` or `--user` or
    /// `--file`
    fn is_zero_valid(&self) -> bool {
        if self.zero && !(self.group | self.groups | self.user | self.file) {
            return false;
        }
        true
    }

    /// Check if `--name` doesn't occour with `--group` or `--groups` or `--user`
    fn is_name_valid(&self) -> bool {
        if self.name && !(self.group | self.groups | self.user) {
            return false;
        }
        true
    }

    /// Check if `--real` doesn't occour with `--group` or `--user`
    fn is_real_valid(&self) -> bool {
        // If real = true and both group and user are false at the same time
        if self.real && !(self.group | self.user) {
            return false;
        }
        true
    }
}

fn default_logic(passwd: &Passwd, sep: char) {
    let groups = match passwd.belongs_to() {
        Ok(gs) => gs,
        Err(err) => {
            eprintln!("id: {}", err);
            process::exit(1);
        },
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

fn group_logic(passwd: &Passwd, flags: IdFlags, sep: char) {
    if flags.name {
        let group = match Group::from_gid(passwd.gid()) {
            Ok(g) => g,
            Err(err) => {
                eprintln!("id: {}", err);
                process::exit(1);
            },
        };
        print!("{}{}", group.name(), sep);
        return;
    }
    print!("{}{}", passwd.gid(), sep);
}

fn user_logic(passwd: &Passwd, flags: IdFlags, sep: char) {
    if flags.name {
        print!("{}{}", passwd.name(), sep);
        return;
    }
    print!("{}{}", passwd.uid(), sep);
}

fn groups_logic(passwd: &Passwd, flags: IdFlags, sep: char) {
    let groups = match passwd.belongs_to() {
        Ok(gs) => gs,
        Err(err) => {
            eprintln!("id: {}", err);
            process::exit(1);
        },
    };

    if flags.name {
        groups.into_iter().for_each(|g| print!("{} ", g.name()));
        print!("{}", sep);
        return;
    }
    groups.into_iter().for_each(|g| print!("{} ", g.id()));
    print!("{}", sep);
}

fn pretty_logic(passwd: &Passwd, sep: char) {
    let groups = match passwd.belongs_to() {
        Ok(gs) => gs,
        Err(err) => {
            eprintln!("id: {}", err);
            process::exit(1);
        },
    };

    print!("uid\t\t{}{}groups\t", passwd.name(), sep);
    groups.into_iter().for_each(|g| print!("{} ", g.name()));
    print!("{}", sep);
}

#[cfg(not(target_os = "freebsd"))]
fn audit_logic() {}

#[cfg(target_os = "freebsd")]
fn audit_logic() {
    match coreutils_core::audit::auditid() {
        Ok(_) => (),
        Err(err) => {
            println!("id: {}", err);
            process::exit(1);
        },
    };
}

#[cfg(not(target_os = "openbsd"))]
fn rtable_logic() {}

#[cfg(target_os = "openbsd")]
fn rtable_logic() {
    use coreutils_core::routing_table::get_routing_table;
    println!("{}", get_routing_table());
}
