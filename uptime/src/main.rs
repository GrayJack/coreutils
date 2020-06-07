use std::process;

#[cfg(target_os = "openbsd")]
use coreutils_core::os::utmp::UtmpSet;
#[cfg(not(target_os = "openbsd"))]
use coreutils_core::os::utmpx::{
    UtmpxKind::{BootTime, UserProcess},
    UtmpxSet as UtmpSet,
};
use coreutils_core::{
    libc::time_t,
    os::{load::load_average, time as ostime},
    time::{OffsetDateTime as DateTime, UtcOffset},
};

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let pretty_flag = matches.is_present("pretty");
    let since_flag = matches.is_present("since");

    let utmps = {
        #[cfg(target_os = "openbsd")]
        match UtmpSet::system() {
            Ok(u) => u,
            Err(err) => {
                eprintln!("uptime: failed to get utmp: {}", err);
                process::exit(1);
            },
        }

        #[cfg(not(target_os = "openbsd"))]
        UtmpSet::system()
    };

    #[cfg(target_os = "openbsd")]
    let num_users = utmps.len();
    #[cfg(not(target_os = "openbsd"))]
    let mut num_users = 0;

    let mut boot_time = DateTime::unix_epoch();
    #[cfg(not(target_os = "openbsd"))]
    for utmp in utmps {
        match utmp.entry_type() {
            BootTime => boot_time = utmp.login_time(),
            UserProcess => num_users += 1,
            _ => continue,
        }
    }

    // If the system doesn't have a BootTime entry on utmps, use this heuristics instead.
    if boot_time == DateTime::unix_epoch() {
        // Get the boot time from the OS. If errors out here, there is nothing left to try
        // so we exit with a error.
        let boot_timeval = ostime::boottime().unwrap_or_else(|err| {
            eprintln!("uptime: Could not retrieve system boot time: {}", err);
            process::exit(1);
        });

        boot_time = DateTime::from_unix_timestamp(boot_timeval.tv_sec)
            .to_offset(UtcOffset::current_local_offset());
    }

    if since_flag {
        let fmt = boot_time.format("%F %T");
        println!("{}", fmt);
        return;
    }

    let up_time = match uptime(boot_time) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("uptime: Could not retrieve system uptime: {}", err);
            process::exit(1);
        },
    };

    if pretty_flag {
        println!("{}", fmt_uptime(up_time, pretty_flag));
        return;
    }

    println!(
        "{} {} {} {}",
        fmt_time(),
        fmt_uptime(up_time, pretty_flag),
        fmt_number_users(num_users),
        fmt_load()
    )
}

fn uptime(boot_time: DateTime) -> Result<time_t, ostime::Error> {
    match ostime::uptime() {
        Ok(t) => Ok(t.tv_sec),
        Err(ostime::Error::TargetNotSupported) => Ok((DateTime::now() - boot_time).whole_seconds()),
        Err(err) => Err(err),
    }
}

fn fmt_time() -> String {
    let now = DateTime::now_local();

    format!(" {:02}:{:02}:{:02}", now.hour(), now.minute(), now.second())
}

fn fmt_load() -> String {
    match load_average() {
        Err(err) => {
            eprintln!("{}", err);
            "".to_string()
        },
        Ok(slice) => {
            let mut msg = String::from("load average: ");
            for item in &slice {
                if (item - slice[2]).abs() < std::f64::EPSILON {
                    msg.push_str(&format!("{:.2}", item));
                } else {
                    msg.push_str(&format!("{:.2}, ", item));
                }
            }
            msg
        },
    }
}

fn fmt_number_users(num_users: usize) -> String {
    match num_users {
        1 => "1 user, ".to_string(),
        n if n > 1 => format!("{} users, ", n),
        _ => "".to_string(),
    }
}

fn fmt_uptime(upsecs: time_t, pretty_flag: bool) -> String {
    let updays = upsecs / 86400;
    let uphours = (upsecs - (updays * 86400)) / 3600;
    let upmins = (upsecs - (updays * 86400) - (uphours * 3600)) / 60;
    if pretty_flag {
        if updays > 0 {
            return format!(
                "up {:1} {}, {:2} {}, {:2} {}",
                updays,
                if updays == 1 { "day" } else { "days" },
                uphours,
                if uphours == 1 { "hour" } else { "hours" },
                upmins,
                if upmins == 1 { "minute" } else { "minutes" },
            );
        } else if uphours > 0 {
            return format!(
                "up {:2} {}, {:2} {}",
                uphours,
                if uphours == 1 { "hour" } else { "hours" },
                upmins,
                if upmins == 1 { "minute" } else { "minutes" },
            );
        } else if upmins > 0 {
            return format!("up {:2} {}", upmins, if upmins == 1 { "minute" } else { "minutes" },);
        } else {
            return format!("up {:2} {}", upsecs, if upmins == 1 { "second" } else { "seconds" },);
        }
    }
    match updays {
        1 => format!("up {:1} day, {:2}:{:02}, ", updays, uphours, upmins),
        n if n > 1 => format!("up {:1} days, {:2}:{:02}, ", updays, uphours, upmins),
        _ => format!("up {:2}:{:02}, ", uphours, upmins),
    }
}
