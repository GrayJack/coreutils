use std::{io, os::unix::fs::MetadataExt, path::PathBuf, process};

#[cfg(target_os = "openbsd")]
use coreutils_core::os::utmp::{Utmp, UtmpSet};
#[cfg(not(target_os = "openbsd"))]
use coreutils_core::os::utmpx::{
    Utmpx,
    UtmpxKind::{BootTime, DeadProcess, InitProcess, LoginProcess, NewTime, RunLevel, UserProcess},
    UtmpxSet as UtmpSet,
};
use coreutils_core::{
    libc::S_IWGRP, os::tty::TTYName, time::OffsetDateTime as DateTime, ByteSlice,
};

use clap::ArgMatches;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let flags = WhoFlags::from_matches(&matches);

    let uts = if matches.is_present("FILE") {
        let file = PathBuf::from(matches.value_of("FILE").unwrap());

        match UtmpSet::from_file(&file) {
            Ok(u) => u,
            #[cfg(not(any(target_os = "openbsd")))]
            Err(_) => UtmpSet::system(),
            #[cfg(any(target_os = "openbsd"))]
            Err(_) => match UtmpSet::system() {
                Ok(uu) => uu,
                Err(err) => {
                    eprintln!("users: failed to get utsp: {}", err);
                    process::exit(1);
                },
            },
        }
    } else {
        #[cfg(any(target_os = "openbsd"))]
        match UtmpSet::system() {
            Ok(u) => u,
            Err(err) => {
                eprintln!("users: failed to get utmp: {}", err);
                process::exit(1);
            },
        }

        #[cfg(not(any(target_os = "openbsd")))]
        UtmpSet::system()
    };

    let mut ut_vec = filter_entries(&uts, flags);
    ut_vec.sort_unstable_by_key(|u| u.login_time());

    if flags.count {
        let mut counter = 0;
        #[cfg(not(target_os = "openbsd"))]
        for ut in ut_vec.iter().filter(|u| u.entry_type() == UserProcess) {
            print!("{} ", ut.user());
            counter += 1;
        }

        #[cfg(target_os = "openbsd")]
        for ut in ut_vec.iter() {
            print!("{} ", ut.user());
            counter += 1;
        }

        println!("\n# users={}", counter);
        return;
    }

    if flags.heading {
        print_header(flags);
    }

    print_info(&ut_vec, flags);
}

#[derive(Debug, Clone, Copy)]
struct WhoFlags {
    boot: bool,
    dead: bool,
    heading: bool,
    login: bool,
    associated_stdin: bool,
    process: bool,
    count: bool,
    run_level: bool,
    short: bool,
    time: bool,
    message: bool,
    idle: bool,
}

impl WhoFlags {
    fn from_matches(matches: &ArgMatches<'_>) -> Self {
        WhoFlags {
            boot: matches.is_present("boot") || matches.is_present("all"),
            dead: matches.is_present("dead") || matches.is_present("all"),
            heading: matches.is_present("heading"),
            login: matches.is_present("login") || matches.is_present("all"),
            associated_stdin: matches.is_present("associated_stdin"),
            process: matches.is_present("process") || matches.is_present("all"),
            count: matches.is_present("count"),
            run_level: matches.is_present("runlevel") || matches.is_present("all"),
            short: matches.is_present("short")
                && !(matches.is_present("idle") || matches.is_present("all")),
            time: matches.is_present("time") || matches.is_present("all"),
            message: matches.is_present("message") || matches.is_present("all"),
            idle: matches.is_present("idle")
                || matches.is_present("users")
                || matches.is_present("all"),
        }
    }

    fn is_all_false(&self) -> bool {
        if let (false, false, false, false, false, false, false, false) = (
            self.boot,
            self.dead,
            self.login,
            self.process,
            self.run_level,
            self.short,
            self.time,
            self.idle,
        ) {
            return true;
        }
        false
    }
}

fn print_header(flags: WhoFlags) {
    if flags.is_all_false() {
        println!("{:<16} {:<10} {:<18} {:<10}", "NAME", "LINE", "TIME", "COMMENT");
    } else if flags.short {
        println!("{:<16} {:<10} {:<18}", "NAME", "LINE", "TIME");
    } else if flags.idle {
        println!("{:<16} {:<10} {:<18} {:<10} {:<10}", "NAME", "LINE", "TIME", "IDLE", "COMMENT");
    } else {
        #[cfg(target_os = "openbsd")]
        println!("{:<16} {:<10} {:<18} {:<10} {:<10}", "NAME", "LINE", "TIME", "IDLE", "COMMENT");
        #[cfg(not(target_os = "openbsd"))]
        println!(
            "{:<16} {:<10} {:<10} {:<18}  {:<10} {:<10}",
            "NAME", "LINE", "PID", "TIME", "IDLE", "COMMENT"
        );
    }
}

#[cfg(target_os = "openbsd")]
fn filter_entries<'a>(uts: &'a UtmpSet, flags: WhoFlags) -> Vec<&'a Utmp> {
    if flags.associated_stdin {
        let curr_tty_name = {
            let tty = match TTYName::new(&io::stdin()) {
                Ok(t) => t,
                Err(err) => {
                    eprintln!("who: failed to get current tty: {}", err);
                    process::exit(1);
                },
            };

            format!("{}", tty).trim_start_matches("/dev/").to_string()
        };

        uts.iter().filter(|u| format!("{}", u.device_name()) == curr_tty_name).collect()
    } else {
        uts.iter().collect()
    }
}

#[cfg(not(target_os = "openbsd"))]
fn filter_entries<'a>(uts: &'a UtmpSet, flags: WhoFlags) -> Vec<&'a Utmpx> {
    let mut uts_user: Vec<_>;
    let mut uts_boot: Vec<_>;
    let mut uts_dead: Vec<_>;
    let mut uts_login: Vec<_>;
    let mut uts_runlv: Vec<_>;
    let mut uts_init: Vec<_>;
    let mut uts_time: Vec<_>;
    let mut ut_vec: Vec<&Utmpx> = Vec::with_capacity(uts.len());

    if flags.associated_stdin {
        let curr_tty_name = {
            let tty = match TTYName::new(&io::stdin()) {
                Ok(t) => t,
                Err(err) => {
                    eprintln!("who: failed to get current tty: {}", err);
                    process::exit(1);
                },
            };

            format!("{}", tty).trim_start_matches("/dev/").to_string()
        };
        let uts_iter = uts.iter().filter(|u| format!("{}", u.device_name()) == curr_tty_name);

        uts_user = uts_iter.clone().filter(|u| u.entry_type() == UserProcess).collect();
        uts_boot = uts_iter.clone().filter(|u| u.entry_type() == BootTime).collect();
        uts_dead = uts_iter.clone().filter(|u| u.entry_type() == DeadProcess).collect();
        uts_login = uts_iter.clone().filter(|u| u.entry_type() == LoginProcess).collect();
        uts_runlv = uts_iter.clone().filter(|u| u.entry_type() == RunLevel).collect();
        uts_init = uts_iter.clone().filter(|u| u.entry_type() == InitProcess).collect();
        uts_time = uts_iter.filter(|u| u.entry_type() == NewTime).collect();
    } else {
        uts_user = uts.iter().filter(|u| u.entry_type() == UserProcess).collect();
        uts_boot = uts.iter().filter(|u| u.entry_type() == BootTime).collect();
        uts_dead = uts.iter().filter(|u| u.entry_type() == DeadProcess).collect();
        uts_login = uts.iter().filter(|u| u.entry_type() == LoginProcess).collect();
        uts_runlv = uts.iter().filter(|u| u.entry_type() == RunLevel).collect();
        uts_init = uts.iter().filter(|u| u.entry_type() == InitProcess).collect();
        uts_time = uts.iter().filter(|u| u.entry_type() == NewTime).collect();
    }

    if flags.is_all_false() {
        ut_vec.append(&mut uts_user);
    } else {
        if flags.short || flags.idle {
            ut_vec.append(&mut uts_user);
        }
        if flags.boot {
            ut_vec.append(&mut uts_boot);
        }
        if flags.dead {
            ut_vec.append(&mut uts_dead);
        }
        if flags.login {
            ut_vec.append(&mut uts_login);
        }
        if flags.run_level {
            ut_vec.append(&mut uts_runlv);
        }
        if flags.process {
            ut_vec.append(&mut uts_init);
        }
        if flags.time {
            ut_vec.append(&mut uts_time);
        }
    }

    ut_vec
}

#[cfg(not(target_os = "openbsd"))]
fn print_info(uts: &[&Utmpx], flags: WhoFlags) {
    if flags.is_all_false() {
        uts.iter().for_each(|u| {
            let (msg, _) = def_status(u);
            println!(
                "{:<12} {:<3} {:<10} {:<18}   {:<10}",
                u.user(),
                if flags.message { msg } else { ' ' },
                u.device_name(),
                u.login_time().format("%Y-%m-%d %H:%M"),
                format!("({})", u.host())
            )
        });
    } else if flags.short {
        uts.iter().for_each(|u| {
            let (msg, _) = def_status(u);
            println!(
                "{:<12} {:<3} {:<10} {:<18}",
                u.user(),
                if flags.message { msg } else { ' ' },
                u.device_name(),
                u.login_time().format("%Y-%m-%d %H:%M"),
            )
        });
    } else if flags.idle {
        uts.iter().for_each(|u| {
            let (msg, idle) = def_status(u);
            println!(
                "{:<12} {:<3} {:<10} {:<18}    {:<10} {:<10}",
                u.user(),
                if flags.message { msg } else { ' ' },
                u.device_name(),
                u.login_time().format("%Y-%m-%d %H:%M"),
                idle,
                format!("({})", u.host())
            )
        });
    } else {
        uts.iter().for_each(|u| {
            let (msg, idle) = def_status(u);
            println!(
                "{:<12} {:<3} {:<10} {:<10} {:<18}    {:<10} {:<10}",
                u.user(),
                if flags.message { msg } else { ' ' },
                u.device_name(),
                u.process_id(),
                u.login_time().format("%Y-%m-%d %H:%M"),
                idle,
                format!("({})", u.host())
            )
        });
    }
}

#[cfg(target_os = "openbsd")]
fn print_info(uts: &[&Utmp], flags: WhoFlags) {
    if flags.is_all_false() {
        uts.iter().for_each(|u| {
            let (msg, _) = def_status(u);
            println!(
                "{:<12} {:<3} {:<10} {:<18}   {:<10}",
                u.user(),
                if flags.message { msg } else { ' ' },
                u.device_name(),
                u.login_time().format("%Y-%m-%d %H:%M"),
                format!("({})", u.host())
            )
        });
    } else if flags.short {
        uts.iter().for_each(|u| {
            let (msg, _) = def_status(u);
            println!(
                "{:<12} {:<3} {:<10} {:<18}",
                u.user(),
                if flags.message { msg } else { ' ' },
                u.device_name(),
                u.login_time().format("%Y-%m-%d %H:%M"),
            )
        });
    } else if flags.idle {
        uts.iter().for_each(|u| {
            let (msg, idle) = def_status(u);
            println!(
                "{:<12} {:<3} {:<10} {:<18}   {:<10} {:<10}",
                u.user(),
                if flags.message { msg } else { ' ' },
                u.device_name(),
                u.login_time().format("%Y-%m-%d %H:%M"),
                idle,
                format!("({})", u.host())
            )
        });
    } else {
        uts.iter().for_each(|u| {
            let (msg, idle) = def_status(u);
            println!(
                "{:<12} {:<3} {:<10} {:<18}   {:<10} {:<10}",
                u.user(),
                if flags.message { msg } else { ' ' },
                u.device_name(),
                u.login_time().format("%Y-%m-%d %H:%M"),
                idle,
                format!("({})", u.host())
            )
        });
    }
}

fn def_status(
    #[cfg(target_os = "openbsd")] utmp: &Utmp, #[cfg(not(target_os = "openbsd"))] utmp: &Utmpx,
) -> (char, String) {
    let mut dev_file = PathBuf::from("/dev");
    let dev_name = match utmp.device_name().to_str() {
        Ok(d) => d,
        Err(err) => {
            eprintln!("who: failed to UTF-8 device name: {}", err);
            process::exit(1);
        },
    };
    dev_file.push(dev_name);

    let msg;
    let last_change;
    if let Ok(meta) = dev_file.metadata() {
        msg = if meta.mode() & (S_IWGRP as u32) == 0 { '-' } else { '+' };
        last_change = meta.atime();
    } else {
        msg = '?';
        last_change = 0;
    };

    let idle = if last_change == 0 {
        "?".to_string()
    } else {
        let now = DateTime::now().timestamp();
        if 0 < last_change && now - 24 * 3600 < last_change && last_change <= now {
            let seconds_idle = now - last_change;
            if seconds_idle < 60 {
                ".".to_string()
            } else {
                format!("{:02}:{:02}", seconds_idle / 3600, (seconds_idle % 3600) / 60)
            }
        } else {
            "old".to_string()
        }
    };

    (msg, idle)
}
