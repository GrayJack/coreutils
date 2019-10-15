use std::fs;

use clap::{load_yaml, App};


fn log<S: Into<String>>(msg: S) {
    println!("mkdir: {}", msg.into());
}

fn main() {
    let yaml = load_yaml!("mkdir.yml");
    let matches = App::from_yaml(yaml).get_matches();
    
    let directories = matches.values_of("DIRECTORY").unwrap();
    let verbose = matches.is_present("verbose");
    let parents = matches.is_present("parents");

    let mkdir = { 
        if parents { 
            fs::create_dir_all
        } else {
            fs::create_dir
        }
    };
    
    for d in directories {
        match mkdir(d) {
            Ok(_) => if verbose { log(format!("created directory '{}'", d)) },
            Err(e) => log(format!("cannot create directory '{}': {}", d, e))
        }
    }
}
