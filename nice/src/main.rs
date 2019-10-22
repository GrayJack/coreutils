#[cfg(target_os = "linux")]
use std::os::raw::c_uint;
use std::{
    os::{raw::c_int, unix::process::CommandExt},
    process::{self, Command},
};

use coreutils_core::priority::{get_priority, set_priority, PRIO_PROCESS};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

#[cfg(target_os = "linux")]
const P_PROCESS: c_uint = PRIO_PROCESS as c_uint;
#[cfg(not(target_os = "linux"))]
const P_PROCESS: c_int = PRIO_PROCESS;

fn main() {
    let yaml = load_yaml!("nice.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let adjustment: c_int = if matches.is_present("N") {
        let str_n = matches.value_of("N").unwrap();
        match str_n.parse() {
            Ok(n) => n,
            Err(err) => {
                eprintln!(r"{} is not a valid number. Err: {}", str_n, err);
                process::exit(125);
            },
        }
    } else {
        10
    };

    let command = matches.value_of("COMMAND").unwrap();
    let args = if matches.is_present("ARGS") {
        matches.values_of("ARGS").unwrap().collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut niceness = match get_priority(P_PROCESS, 0) {
        Ok(nice) => nice,
        Err(err) => {
            eprintln!("nice: {}", err);
            drop(args);
            process::exit(125);
        },
    };

    niceness += adjustment;

    if let Err(err) = set_priority(P_PROCESS, 0, niceness) {
        eprintln!("nice: {}", err);
        drop(args);
        process::exit(125);
    }

    let err = Command::new(command).args(args).exec();

    if err.raw_os_error().unwrap() as c_int == 2
    // ENOENT
    {
        eprintln!("nice: '{}': {}", command, err);
        process::exit(127);
    } else {
        eprintln!("nice: {}", err);
        process::exit(126);
    }
}
