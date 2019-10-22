use std::{
    fs::File,
    io::{self, Read},
    process,
};

use coreutils_core::{
    libc::time_t,
    load::load_average,
    time,
    utmpx::{
        UtmpxSet,
        UtmpxType::{BootTime, UserProcess},
    },
};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("uptime.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let pretty_flag = matches.is_present("pretty");
    let since_flag = matches.is_present("since");

    let utmpxs = UtmpxSet::system();

    let mut num_users = 0;
    let mut boot_time = time::empty_tm();
    for utmpx in utmpxs {
        match utmpx.utype() {
            BootTime => boot_time = utmpx.login_time(),
            UserProcess => num_users += 1,
            _ => continue,
        }
    }

    let up_time = match uptime(boot_time.to_timespec().sec) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("uptime: could not retrieve system uptime: {}", err);
            process::exit(1);
        },
    };

    if since_flag {
        let fmt = match boot_time.strftime("%F %T") {
            Ok(f) => f,
            Err(err) => {
                eprintln!("uptime: failed to format time: {}", err);
                process::exit(1);
            },
        };
        println!("{}", fmt);
        return;
    }

    if pretty_flag {
        println!("{}", fmt_uptime(up_time / 100, pretty_flag));
        return;
    }

    println!(
        "{} {} {} {}",
        fmt_time(),
        fmt_uptime(up_time / 100, pretty_flag),
        fmt_number_users(num_users),
        fmt_load()
    )
}

fn uptime(boot_time: time_t) -> io::Result<time_t> {
    let mut file_uptime = String::new();

    if let Ok(mut f) = File::open("/proc/uptime") {
        f.read_to_string(&mut file_uptime)?;
        file_uptime
            .split_whitespace()
            .take(1)
            .collect::<String>()
            .replace(".", "")
            .parse()
            .or_else(|_| Err(io::Error::last_os_error()))
    } else {
        let now = time::get_time().sec as time_t;
        Ok(now - boot_time)
    }
}

fn fmt_time() -> String {
    let now = time::now();

    format!(" {:02}:{:02}:{:02}", now.tm_hour, now.tm_min, now.tm_sec)
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
                if item == &slice[2] {
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
        _ => format!("up  {:2}:{:02}, ", uphours, upmins),
    }
}
