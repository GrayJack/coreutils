extern crate chrono;

use std::{fmt, io, process};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};

use clap::{load_yaml, App, ArgMatches};
use std::{io::ErrorKind, path::Path};

fn main() {
    let yaml = load_yaml!("date.yml");
    let matches = App::from_yaml(yaml).get_matches();

    match date(&matches) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("date: {}", e);
            process::exit(1);
        },
    };
}

/// prints the local datetime.
/// If `is_utc` is true, the datetime is printed in universal time.
/// If `date` is Some it will get parsed and printed out instead of the current datetime.
fn date<'a>(args: &ArgMatches) -> Result<(), &'a str> {
    let is_rfc2822 = args.is_present("RFC2822");
    let is_utc = args.is_present("utc");
    let is_date = args.is_present("date");
    let is_read = args.is_present("read");

    let is_outputformat = args.is_present("outputformat");
    let is_convert = args.is_present("convert");
    let is_format = args.is_present("format");

    let mut datetime: DateTime<Local> = Local::now();

    if is_date {
        let date_str = args.value_of("date").unwrap();
        match read_date(date_str) {
            Ok(date) => datetime = date,
            Err(e) => return Err(e),
        }
        if !is_convert {
            // TODO this should usually set OS' datetime if -j is not given
            unimplemented!();
            return Ok(());
        }
    }

    if is_format {
        let mut values = args.values_of("format").unwrap();
        let input_fmt = values.next().unwrap();
        let new_date = values.next().unwrap();

        match parse_date(new_date, input_fmt) {
            Ok(date) => datetime = date,
            Err(e) => return Err(e),
        }

        if !is_convert {
            // TODO this should usually set OS' datetime if -j is not given
            unimplemented!();
            return Ok(());
        }
    }


    if is_read {
        let date_str = args.value_of("read").unwrap();
        match read(date_str) {
            Ok(date) => datetime = date,
            Err(e) => return Err(e),
        }
    }

    if is_outputformat {
        format(datetime, args.value_of("outputformat").unwrap(), is_utc);
    } else if is_rfc2822 {
        format_rfc2822(datetime, is_utc);
    } else {
        format_standard(datetime, is_utc);
    }

    Ok(())
}

/// Reads datetime from `input`. Could be seconds or a filepath.
fn read<'a>(input: &str) -> Result<DateTime<Local>, &'a str> {
    let parsed: Result<i32, _> = input.trim().parse();
    let result = match parsed {
        Ok(_) => parse_seconds(input.trim()),
        Err(_) => parse_file(input),
    };

    if let Ok(date) = result {
        return Ok(date);
    } else {
        return Err("illegal date time format");
    }
}

/// Parses datetime from `date_str` with format `[[[[[cc]yy]mm]dd]HH]MM[.ss]`.
fn read_date<'a>(date_str: &str) -> Result<DateTime<Local>, &'a str> {
    let format = build_parse_format(date_str);
    parse_date(date_str, &format)
}

/// Parsed datetime from `date_str` with `format`.
fn parse_date<'a>(date_str: &str, format: &str) -> Result<DateTime<Local>, &'a str> {
    match parse_datetime_from_str(date_str, &format) {
        Ok(d) => Ok(d),
        Err(_) => Err("illegal date time format"),
    }
}

/// Return the local DateTime
fn parse_seconds(seconds: &str) -> Result<DateTime<Local>, io::Error> {
    match NaiveDateTime::parse_from_str(seconds, "%s") {
        Ok(date) => {
            // let local = TimeZone::from_local_datetime(&Local, &date).unwrap();
            let local = TimeZone::from_utc_datetime(&Local, &date);
            Ok(local)
        },
        Err(e) => Err(io::Error::new(ErrorKind::InvalidInput, e)),
    }
}

/// Returns the modified date of `filename`.
/// Returns `NotFound` if `filename` could not be found.
fn parse_file(filename: &str) -> Result<DateTime<Local>, io::Error> {
    let path = Path::new(filename);

    if path.exists() {
        let metadata = path.metadata().unwrap();
        let modified = metadata.modified().unwrap();
        let datetime: DateTime<Local> = DateTime::from(modified);

        return Ok(datetime);
    } else {
        return Err(io::Error::from(ErrorKind::NotFound));
    }
}

/// Builds the correct datetime format string to parse `date`.
fn build_parse_format(date: &str) -> String {
    // format is [[[[[cc]yy]mm]dd]HH]MM[.ss]
    let mut format = vec![' ', ' ', ' ', ' ', ' ', ' ', ' '];
    let mut len = date.chars().count();

    if date.contains(".") {
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
        if !chr.eq(&' ') {
            format_str.push_str("%");
            format_str.push_str(&chr.to_string());
        }
    }

    if format[6] != ' ' {
        format_str.push_str(".%S")
    }

    format_str
}

/// This function parses `datetime` of given `format`. If `datetime` is not enough for a
/// unique DateTime it uses die values of today.
fn parse_datetime_from_str<'a>(datetime: &str, format: &str) -> Result<DateTime<Local>, &'a str> {
    let result = NaiveDateTime::parse_from_str(datetime, format);

    // TODO Chrono's parse of strftime throws an error if it can not create an unique
    // datetime. What we want is that it fills the missing data with data of
    // DateTime::now()

    match result {
        Ok(datetime) => Ok(TimeZone::from_utc_datetime(&Local, &datetime)),
        Err(_) => Err("could not parse datetime"),
    }
}

/// displays `datetime` in rfc2822 format
fn format_rfc2822<Tz: TimeZone>(datetime: DateTime<Tz>, is_utc: bool)
where Tz::Offset: fmt::Display {
    let format_str = "%a, %d %b %Y %T %z";
    format(datetime, format_str, is_utc);
}

/// displays `datetime` standard format `"%a %b %e %k:%M:%S %Z %Y"`
fn format_standard<Tz: TimeZone>(datetime: DateTime<Tz>, is_utc: bool)
where Tz::Offset: fmt::Display {
    let format_str = "%a %b %e %k:%M:%S %Z %Y"; // <- %Z should print the name of the timezone (only works for UTC)
                                                // problem is in chrono lib: https://github.com/chronotope/chrono/issues/288
    format(datetime, format_str, is_utc);
}

/// displays `datetime` with given `output_format`
fn format<Tz: TimeZone>(datetime: DateTime<Tz>, output_format: &str, is_utc: bool)
where Tz::Offset: fmt::Display {
    if is_utc {
        println!("{}", datetime.with_timezone(&Utc).format(output_format));
    } else {
        println!("{}", datetime.with_timezone(&Local).format(output_format));
    }
}
