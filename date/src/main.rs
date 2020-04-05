use std::{path::Path, str::FromStr};

use coreutils_core::time::{
    Date, Duration, OffsetDateTime as DateTime, PrimitiveDateTime, Time, UtcOffset,
};

use clap::ArgMatches;

mod cli;

const DEFAULT_FMT_OUT: &str = "%a %b %d %H:%M:%S %z %Y";
const RFC_2822_FMT: &str = "%a, %d %b %Y %T %z";

fn main() {
    let matches = cli::create_app().get_matches();

    if let Err(err) = date(&matches) {
        eprintln!("date: {}", err);
        std::process::exit(1);
    }
}

fn date(matches: &ArgMatches) -> Result<(), String> {
    let is_iso8601 = matches.is_present("iso8601");
    let is_rfc2822 = matches.is_present("rfc2822");
    let is_rfc3339 = matches.is_present("rfc3339");
    let is_set = matches.is_present("set") && !matches.is_present("no_set");

    let utc_off =
        if matches.is_present("utc") { UtcOffset::UTC } else { UtcOffset::current_local_offset() };

    let (out_fmt, date_str) = {
        match (matches.is_present("OPERAND"), matches.is_present("DATE")) {
            (true, false) => {
                let value = matches.value_of("OPERAND").unwrap();

                if value.starts_with('+') {
                    (&value[1..], "now")
                } else if is_rfc2822 {
                    (RFC_2822_FMT, value)
                } else if is_iso8601 {
                    // Ok to unwrap cause we just checked that it's present and it has default
                    // value.
                    let fmt_str = matches.value_of("iso8601").unwrap();
                    (iso8601_format_str(fmt_str), value)
                } else if is_rfc3339 {
                    // Ok to unwrap cause we just checked that it's present and it has default
                    // value.
                    let fmt_str = matches.value_of("rfc3339").unwrap();
                    (rfc3339_format_str(fmt_str), value)
                } else {
                    (DEFAULT_FMT_OUT, value)
                }
            },
            (true, true) => {
                let op = matches.value_of("OPERAND").unwrap();
                let date = matches.value_of("DATE").unwrap();

                if !op.starts_with('+') {
                    return Err("Operand format is invalid: Must have '+' at start".to_string());
                }

                (&op[1..], date)
            },
            (false, false) => {
                if is_rfc2822 {
                    (RFC_2822_FMT, "now")
                } else if is_iso8601 {
                    // Ok to unwrap cause we just checked that it's present and it has default
                    // value.
                    let iso_str = matches.value_of("iso8601").unwrap();
                    (iso8601_format_str(iso_str), "now")
                } else if is_rfc3339 {
                    // Ok to unwrap cause we just checked that it's present and it has default
                    // value.
                    let fmt_str = matches.value_of("rfc3339").unwrap();
                    (rfc3339_format_str(fmt_str), "now")
                } else {
                    (DEFAULT_FMT_OUT, "now")
                }
            },
            // SAFETY: Cannot happen, because it will always get the first argument as "OPERAND"
            // We fix that on the (true, false) case.
            (false, true) => unreachable!(),
        }
    };

    let date = build_datetime(date_str, utc_off, matches.value_of("reference"))?;

    if is_set {
        set_os_time(date)?;
    }

    println!("{}", date.format(out_fmt));
    Ok(())
}

/// Build a [`DateTime`] from a `date_str`.
///
/// The `date_str` format is `[[[[[CC]YY]MM]DD]hh]mm[.SS]`
fn build_datetime(
    date_str: &str, utc_off: UtcOffset, ref_val: Option<&str>,
) -> Result<DateTime, String> {
    // If read_input is Some, that means that read flag was set. Else use now
    let now = match ref_val {
        Some(s) => reference_datetime(s, utc_off)?,
        None => DateTime::now().to_offset(utc_off),
    };

    if date_str == "now" {
        return Ok(now);
    }

    let mut len = date_str.chars().count();
    let chars: Vec<_> = date_str.chars().collect();

    let sec = if date_str.contains('.') {
        let index = date_str.split('.').nth(0).unwrap().len() + 1;
        let sec = match &chars[index..] {
            [] => return Err("No values after '.'".to_string()),
            [_] => return Err("Only one digit: Must have two digits after '.'".to_string()),
            [s1, s2] => parse_datetime_values(&[*s1, *s2])?,
            _ => return Err("Too many digits: Must have two digits after '.'".to_string()),
        };
        len -= 3;
        sec
    } else {
        now.second()
    };

    match &chars[..len] {
        [m1, m2] => {
            let min = parse_datetime_values(&[*m1, *m2])?;
            let time = build_time(now.hour(), min, sec, now.nanosecond())?
                - Duration::seconds(now.offset().as_seconds().into());
            Ok(PrimitiveDateTime::new(now.date(), time).assume_utc().to_offset(now.offset()))
        },
        [h1, h2, m1, m2] => {
            let hour = parse_datetime_values(&[*h1, *h2])?;
            let min = parse_datetime_values(&[*m1, *m2])?;
            let time = build_time(hour, min, sec, now.nanosecond())?
                - Duration::seconds(now.offset().as_seconds().into());
            Ok(PrimitiveDateTime::new(now.date(), time).assume_utc().to_offset(now.offset()))
        },
        [d1, d2, h1, h2, m1, m2] => {
            let day = parse_datetime_values(&[*d1, *d2])?;
            let hour = parse_datetime_values(&[*h1, *h2])?;
            let min = parse_datetime_values(&[*m1, *m2])?;
            let date = build_date(now.year(), now.month(), day)?;
            let time = build_time(hour, min, sec, now.nanosecond())?
                - Duration::seconds(now.offset().as_seconds().into());
            Ok(PrimitiveDateTime::new(date, time).assume_utc().to_offset(now.offset()))
        },
        [mo1, mo2, d1, d2, h1, h2, m1, m2] => {
            let month = parse_datetime_values(&[*mo1, *mo2])?;
            let day = parse_datetime_values(&[*d1, *d2])?;
            let hour = parse_datetime_values(&[*h1, *h2])?;
            let min = parse_datetime_values(&[*m1, *m2])?;
            let date = build_date(now.year(), month, day)?;
            let time = build_time(hour, min, sec, now.nanosecond())?
                - Duration::seconds(now.offset().as_seconds().into());
            Ok(PrimitiveDateTime::new(date, time).assume_utc().to_offset(now.offset()))
        },
        [y1, y2, mo1, mo2, d1, d2, h1, h2, m1, m2] => {
            let cc = now.format("%C").chars().collect::<Vec<_>>();
            let year = parse_datetime_values(&[cc[0], cc[1], *y1, *y2])?;
            let month = parse_datetime_values(&[*mo1, *mo2])?;
            let day = parse_datetime_values(&[*d1, *d2])?;
            let hour = parse_datetime_values(&[*h1, *h2])?;
            let min = parse_datetime_values(&[*m1, *m2])?;
            let date = build_date(year, month, day)?;
            let time = build_time(hour, min, sec, now.nanosecond())?
                - Duration::seconds(now.offset().as_seconds().into());
            Ok(PrimitiveDateTime::new(date, time).assume_utc().to_offset(now.offset()))
        },
        [c1, c2, y1, y2, mo1, mo2, d1, d2, h1, h2, m1, m2] => {
            let year = parse_datetime_values(&[*c1, *c2, *y1, *y2])?;
            let month = parse_datetime_values(&[*mo1, *mo2])?;
            let day = parse_datetime_values(&[*d1, *d2])?;
            let hour = parse_datetime_values(&[*h1, *h2])?;
            let min = parse_datetime_values(&[*m1, *m2])?;
            let date = build_date(year, month, day)?;
            let time = build_time(hour, min, sec, now.nanosecond())?
                - Duration::seconds(now.offset().as_seconds().into());
            Ok(PrimitiveDateTime::new(date, time).assume_utc().to_offset(now.offset()))
        },
        _ => Err("Invalid digits".to_string()),
    }
}

/// Reads datetime from `input`. Could be seconds or a filepath.
fn reference_datetime(input: &str, utc_off: UtcOffset) -> Result<DateTime, String> {
    // First try to parse as a number, if it fails, treat as a file
    match input.trim().parse::<i64>() {
        Ok(sec) => Ok(DateTime::from_unix_timestamp(sec).to_offset(utc_off)),
        Err(p_err) => match datetime_from_file(input, utc_off) {
            Ok(d) => Ok(d),
            Err(f_err) => Err(format!(
                "Invalid read input: Neither a file or seconds: {} AND {}",
                f_err, p_err
            )),
        },
    }
}

/// Returns the last modified date of `filename`.
fn datetime_from_file(filename: impl AsRef<Path>, utc_off: UtcOffset) -> std::io::Result<DateTime> {
    let path = filename.as_ref();

    let metadata = path.metadata()?;
    let modified = metadata.modified()?;

    Ok(DateTime::from(modified).to_offset(utc_off))
}

/// Parses a slice of [`char`]s and return a value of a type that implements [`FromStr`].
fn parse_datetime_values<T: FromStr>(chars: &[char]) -> Result<T, String>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match chars.iter().collect::<String>().parse::<T>() {
        Ok(x) => Ok(x),
        Err(err) => Err(format!("Failed to parse date string: {}", err)),
    }
}

/// Build a [`Date`]. Convenience method that resturn the same type of error as
/// [`build_datetime`].
fn build_date(year: i32, month: u8, day: u8) -> Result<Date, String> {
    match Date::try_from_ymd(year, month, day) {
        Ok(d) => Ok(d),
        Err(err) => Err(format!("Invalid date digits: {}", err)),
    }
}

/// Build a [`Time`]. Convenience method that resturn the same type of error as
/// [`build_datetime`].
fn build_time(hour: u8, min: u8, sec: u8, nano: u32) -> Result<Time, String> {
    match Time::try_from_hms_nano(hour, min, sec, nano) {
        Ok(t) => Ok(t),
        Err(err) => Err(format!("Invalid time digits: {}", err)),
    }
}

/// Returns the fortmat string acording to possible ISO8601 values.
fn iso8601_format_str(value: &str) -> &str {
    match value {
        "date" | "" => "%F",
        "hour" => "%FT%H",
        "hours" => "%FT%H%z",
        "minute" => "%FT%H:%M",
        "minutes" => "%FT%H:%M%z",
        "second" => "%FT%H:%M:%S",
        "seconds" => "%FT%H:%M:%S%z",
        // SAFETY: Clap ensures that only the above values are used
        _ => unreachable!(),
    }
}

/// Returns the fortmat string acording to possible RFC3339 values.
fn rfc3339_format_str(value: &str) -> &str {
    match value {
        "date" | "" => "%F",
        "hour" => "%F %H",
        "hours" => "%F %H%z",
        "minute" => "%F %H:%M",
        "minutes" => "%F %H:%M%z",
        "second" => "%F %H:%M:%S",
        "seconds" => "%F %H:%M:%S%z",
        "nanosecond" => "%F %H:%M:%S.%N",
        "nanoseconds" | "ns" => "%F %H:%M:%S.%N%z",
        // SAFETY: Clap ensures that only the above values are used
        _ => unreachable!(),
    }
}

/// Sets the os datetime to `datetime`
fn set_os_time(datetime: DateTime) -> Result<(), String> {
    use coreutils_core::os::{time::set_time_of_day, Susec, Time, TimeVal};

    let time =
        TimeVal { tv_sec: datetime.timestamp() as Time, tv_usec: datetime.microsecond() as Susec };

    match set_time_of_day(time) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Failed to set date: {}", err)),
    }
}
