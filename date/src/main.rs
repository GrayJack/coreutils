extern crate chrono;

use std::{
    io::{self, Write},
    process,
};

use chrono::{DateTime, Local, NaiveDateTime, Utc};

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("date.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let is_utc = matches.is_present("utc");
    let date_value = matches.value_of("date");

    match date(is_utc, date_value) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Date: Failed to write to stdout.\n{}", e);
            process::exit(1);
        }
    };
}

/// prints the local datetime.
/// If `is_utc` is true, the datetime is printed in universal time.
/// If `date` is Some it will get parsed and printed out instead of the current datetime.
fn date(is_utc: bool, date: Option<&str>) -> io::Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();

    let naive_datetime: NaiveDateTime;

    if let Some(date_str) = date {
        let parsed_date = NaiveDateTime::parse_from_str(date_str, "%m%d%H%M%Y");

        match parsed_date {
            Ok(d) => naive_datetime = d,
            Err(e) => {
                eprintln!("Date: Could not parse given date.\n{}", e);
                process::exit(1);
            }
        }
    } else {
        if is_utc {
            let utc_datetime: DateTime<Utc> = Utc::now();
            naive_datetime = utc_datetime.naive_utc();
        } else {
            let local_datetime: DateTime<Local> = Local::now();
            naive_datetime = local_datetime.naive_local();
        }
    }

    write!(output, "{}", naive_datetime.format("%a %b %e %k:%M:%S %Y"))?;

    Ok(())
}
