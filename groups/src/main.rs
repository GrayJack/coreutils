use coreutils_core::group::{get_groups, Group};

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("groups.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let filter_name = matches.is_present("NAME");
    let id = matches.is_present("id");

    let name = if filter_name {
        matches.value_of("NAME").unwrap()
    } else { "" };

    let groups = match get_groups() {
        Ok(gs) => gs,
        _ => Vec::new(),
    };

    let user_group = if filter_name {
        Group::from_name(name)
    } else {
        Group::new()
    };

    if !groups.is_empty() {
        if filter_name {
            if id {
                for group in groups.iter().filter(|g| g.mem() == name) {
                    print!("{}:{} ", group.name(), group.id());
                }
                print!("{}:{} ", user_group.name(), user_group.id());
            } else {
                for group in groups.iter().filter(|g| g.mem() == name) {
                    print!("{} ", group.name());
                }
                print!("{} ", user_group.name());
            }
        } else if id {
            for group in groups {
                print!("{}:{} ", group.name(), group.id());
            }
        } else {
            for group in groups {
                print!("{} ", group.name());
            }
        }
    }
    println!();
}
