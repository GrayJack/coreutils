use std::{path::Path, str::FromStr};

use clap::ArgMatches;
use coreutils_core::time::{
    Date, Duration, OffsetDateTime as DateTime, PrimitiveDateTime, Time, UtcOffset,
};

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
    let iso8601 = matches.value_of("iso8601");
    let rfc2822 = matches.value_of("rfc2822");
    let rfc3339 = matches.value_of("rfc3339");
    #[cfg(not(target_os = "haiku"))]
    let is_set = matches.is_present("set") && !matches.is_present("no_set");

    let utc_off = if matches.is_present("utc") {
        UtcOffset::UTC
    } else {
        UtcOffset::try_current_local_offset().unwrap_or_else(|err| {
            eprintln!("uptime: {}: UTC offset default value will be used (offset zero)", err);
            UtcOffset::UTC
        })
    };

    let (out_fmt, date_str) = {
        match (matches.value_of("OPERAND"), matches.value_of("DATE")) {
            (Some(operand), None) => {
                if let Some(s) = operand.strip_prefix('+') {
                    (s, "now")
                } else if rfc2822.is_some() {
                    (RFC_2822_FMT, operand)
                } else if let Some(fmt_str) = iso8601 {
                    (iso8601_format_str(fmt_str), operand)
                } else if let Some(fmt_str) = rfc3339 {
                    (rfc3339_format_str(fmt_str), operand)
                } else {
                    (DEFAULT_FMT_OUT, operand)
                }
            },
            (Some(operand), Some(date)) => match operand.strip_prefix('+') {
                Some(op) => (op, date),
                None => return Err("Operand format is invalid: Must have '+' at start".to_string()),
            },
            (None, None) => {
                if rfc2822.is_some() {
                    (RFC_2822_FMT, "now")
                } else if let Some(iso_str) = iso8601 {
                    (iso8601_format_str(iso_str), "now")
                } else if let Some(fmt_str) = rfc3339 {
                    (rfc3339_format_str(fmt_str), "now")
                } else {
                    (DEFAULT_FMT_OUT, "now")
                }
            },
            // SAFETY: Cannot happen, because it will always get the first argument as "OPERAND"
            // We fix that on the (true, false) case.
            (None, Some(_)) => unreachable!(),
        }
    };

    let date = build_datetime(date_str, utc_off, matches.value_of("reference"))?;

    #[cfg(not(target_os = "haiku"))]
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
        None => DateTime::now_utc().to_offset(utc_off),
    };

    if date_str == "now" {
        return Ok(now);
    }

    let mut len = date_str.chars().count();
    let chars: Vec<_> = date_str.chars().collect();

    let sec = if date_str.contains('.') {
        let index = date_str.split('.').next().unwrap().len() + 1;
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
    chars
        .iter()
        .collect::<String>()
        .parse::<T>()
        .map_err(|err| format!("Failed to parse date string: {}", err))
}

/// Build a [`Date`]. Convenience method that resturn the same type of error as
/// [`build_datetime`].
fn build_date(year: i32, month: u8, day: u8) -> Result<Date, String> {
    Date::try_from_ymd(year, month, day).map_err(|err| format!("Invalid date digits: {}", err))
}

/// Build a [`Time`]. Convenience method that resturn the same type of error as
/// [`build_datetime`].
fn build_time(hour: u8, min: u8, sec: u8, nano: u32) -> Result<Time, String> {
    Time::try_from_hms_nano(hour, min, sec, nano)
        .map_err(|err| format!("Invalid time digits: {}", err))
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
#[cfg(not(target_os = "haiku"))]
fn set_os_time(datetime: DateTime) -> Result<(), String> {
    use coreutils_core::os::{time::set_time_of_day, Susec, Time, TimeVal};

    let time = TimeVal {
        tv_sec: datetime.unix_timestamp() as Time,
        tv_usec: datetime.microsecond() as Susec,
    };

    match set_time_of_day(time) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Failed to set date: {}", err)),
    }
}
