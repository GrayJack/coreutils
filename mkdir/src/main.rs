use std::fs;
use std::os::unix::fs::PermissionsExt;

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
    let has_mode = matches.is_present("mode");
    let mode = matches.value_of("mode").unwrap();

    let mkdir = { 
        if parents { 
            fs::create_dir_all
        } else {
            fs::create_dir
        }
    };
    
    for d in directories {
        match mkdir(d) {
            Ok(_) => {
                if verbose { log(format!("created directory '{}'", d)) };
                if has_mode {
                    match fs::metadata(d) {
                        Ok(v) => {
                            let mut perms = v.permissions();
                            let umode: u32 = mode.parse().unwrap();
                            perms.set_mode(umode);
                        }
                        Err(e) => {
                            log(format!("{}", e));
                        }
                    }
                }
            }
            Err(e) => log(format!("cannot create directory '{}': {}", d, e))
        }
    }
}
