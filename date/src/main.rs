extern crate chrono;

use std::{
    fmt,
    io,
    process,
};

use chrono::{Local, DateTime, TimeZone, Utc, NaiveDateTime, FixedOffset, Date, ParseResult};

use clap::{load_yaml, App, ArgMatches};
use std::string::ParseError;
use std::path::Path;
use std::error::Error;
use std::io::ErrorKind;
use std::str::FromStr;

fn main() {
    let yaml = load_yaml!("date.yml");
    let matches = App::from_yaml(yaml).get_matches();

    match date(&matches) {
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
fn date(args: &ArgMatches) -> io::Result<()> {
    let is_rfc2822 = args.is_present("RFC2822");
    let is_utc = args.is_present("utc");
    let is_date = args.is_present("date");
    let is_read = args.is_present("read");

    if is_read || is_date {
        let mut datetime: Option<DateTime<FixedOffset>> = Option::None;
        if is_date {
            let date_str = args.value_of("date").unwrap();
            if let Ok(format) = build_parse_format(date_str, date_str.chars().count()) {
                match DateTime::parse_from_str(date_str, &format) {
                    Ok(d) => datetime = Some(d),
                    Err(e) => return Err(io::Error::from(ErrorKind::InvalidInput))
                }
            }
        } else {
            let date_str = args.value_of("read").unwrap();
            let parsed: Result<i32, _> = date_str.trim().parse();
            let result = match parsed {
                Ok(_) => parse_seconds(date_str.trim()),
                Err(_) => parse_file(date_str)
            };

            if let Ok(date) = result {
                datetime = Some(date);
            }
        }

        if is_rfc2822 {
            format_rfc2822(datetime.unwrap());
        } else {
            format_standard(datetime.unwrap());
        }
    } else if is_utc {
        let datetime = Utc::now();

        if is_rfc2822 {
            format_rfc2822(datetime);
        } else {
            format_standard(datetime);
        }
    } else {
        let datetime = Local::now();

        if is_rfc2822 {
            format_rfc2822(datetime);
        } else {
            format_standard(datetime);
        }
    }

    Ok(())
}

fn parse_seconds(seconds: &str) -> Result<DateTime<FixedOffset>, io::Error> {
    match DateTime::parse_from_str(seconds, "%S") {
        Ok(date) => Ok(date),
        Err(e) => Err(io::Error::new(ErrorKind::InvalidInput, e))
    }
}

fn parse_file(filename: &str) -> Result<DateTime<FixedOffset>, io::Error> {
    let path = Path::new(filename) ;
    if path.exists() {
        let metadata = path.metadata().unwrap();
        let modified = metadata.modified().unwrap();
        println!("System time is {:?} ", modified);
        let datetime: DateTime<Utc> = DateTime::from(modified); // TODO this should be local time
        let datetime_fixed: DateTime<FixedOffset> = DateTime::from(datetime);
        return Ok(datetime_fixed);
    } else {
        return Err(io::Error::from(ErrorKind::NotFound));
    }
}

fn build_parse_format(date: &str, date_str_len: usize) -> Result<String, ()> {
    // format is [[[[[cc]yy]mm]dd]HH]MM[.ss]
    let mut result: Result<String, ()> = Err(());
    let mut format = vec![' ', ' ', ' ', ' ', ' ', ' ', ' '];
    let mut len = date_str_len;

    if date.contains("\\.") {
        format[6] = 'S';
        len -= 3;
    }

    if len >= 2 {
        format[5] = 'M';
    }
    if len >= 4 {
        format[4] = 'H';
    }
    if len >= 6 {
        format[3] = 'd';
    }
    if len >= 8 {
        format[2] = 'm';
    }
    if len >= 10 {
        format[1] = 'y';
    }
    if len >= 12 {
        format[0] = 'C';
    }

    let mut format_str = String::new();
    let spliced_format = format[..6].iter();

    for chr in spliced_format {
        format_str.push_str("%");
        format_str.push_str(&chr.to_string());
    }

    if format[6] != ' ' {
        format_str.push_str(".%S")
    }

    Ok(format_str)
}

/// displays `datetime` in rfc2822 format
fn format_rfc2822<Tz: TimeZone>(datetime: DateTime<Tz>) where Tz::Offset : fmt::Display {
    let format_str = "%a, %d %b %Y %T %z";
    format(datetime, format_str);
}

/// displays `datetime` standard format `"%a %b %e %k:%M:%S %Z %Y"`
fn format_standard<Tz: TimeZone>(datetime: DateTime<Tz>) where Tz::Offset : fmt::Display {
    let format_str = "%a %b %e %k:%M:%S %Z %Y"; // <- %Z should print the name of the timezone (only works for UTC)
    // problem is in chrono lib: https://github.com/chronotope/chrono/issues/288
    format(datetime, format_str);
}

/// displays `datetime` with given `output_format`
fn format<Tz: TimeZone >(datetime: DateTime<Tz>, output_format: &str) where Tz::Offset : fmt::Display {
    println!("{}", datetime.format(output_format));
}