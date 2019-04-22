use coreutils_core::group::get_groups;

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("groups.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // TODO: Do the logig for NAME after a update of coreutils_core
    let _name = match matches.value_of("NAME") {
        Some(n) => n,
        None => ""
    };

    let groups = match get_groups() {
        Ok(gs) => gs,
        _ => Vec::new()
    };

    let id = matches.is_present("id");
    if !groups.is_empty() {
        if id {
            for group in groups {
                print!("{}:{} ", group.name(), group.id() );
            }
        } else {
            for group in groups {
                print!("{} ", group.name() );
            }
        }
    }
    println!();
}
