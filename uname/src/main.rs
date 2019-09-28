use std::process;

use coreutils_core::{
    consts::{HOST_OS, MACHINE_ARCH},
    utsname::UtsName,
};

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("uname.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let all_flag = matches.is_present("all");
    let sysname_flag = matches.is_present("sysname");
    let nodename_flag = matches.is_present("nodename");
    let release_flag = matches.is_present("release");
    let osversion_flag = matches.is_present("osversion");
    let machine_flag = matches.is_present("machine");
    let processor_flag = matches.is_present("processor");
    let os_flag = matches.is_present("os");

    let uts_name = match UtsName::new() {
        Ok(uname) => uname,
        Err(err) => {
            eprintln!("uname: {}", err);
            process::exit(1);
        }
    };

    if let (false, false, false, false, false, false, false, false) = (
        all_flag,
        sysname_flag,
        nodename_flag,
        release_flag,
        osversion_flag,
        machine_flag,
        os_flag,
        processor_flag,
    ) {
        println!("{}", uts_name.system_name());
        return
    }

    if all_flag {
        println!("{} {}", uts_name, HOST_OS);
        return
    }

    if sysname_flag {
        print!("{} ", uts_name.system_name());
    }

    if nodename_flag {
        print!("{} ", uts_name.node_name());
    }

    if release_flag {
        print!("{} ", uts_name.release());
    }

    if osversion_flag {
        print!("{} ", uts_name.version());
    }

    if machine_flag {
        print!("{} ", uts_name.machine());
    }

    if processor_flag {
        print!("{} ", MACHINE_ARCH);
    }

    if os_flag {
        print!("{} ", HOST_OS);
    }

    println!();
}
