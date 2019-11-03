use std::{
    env::current_dir,
    process,
};

use clap::{load_yaml, App, ArgMatches};

struct Nl {

}

impl Nl {
    fn from_matches(matches: &ArgMatches) -> Self {
        Nl {}
    }
}

fn main() {
    let yaml = load_yaml!("nl.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let mut nl = Nl::from_matches(&matches);
    let cwd = match current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("nl: error reading current working directory: {}", err);
            process::exit(1);
        },
    };
}
