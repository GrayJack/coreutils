use std::process;

use clap::{load_yaml, App, AppSettings};

#[derive(Copy, Clone, Debug)]
pub struct Seq {
    current: f64,
    stop: f64,
    step: f64,
}

impl Iterator for Seq {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.step.is_sign_positive() && self.current <= self.stop)
            || (self.step.is_sign_negative() && self.current >= self.stop)
        {
            let result = self.current;

            self.current += self.step;

            Some(result)
        } else {
            None
        }
    }
}

fn parse_number(text: &str) -> Result<f64, ()> {
    text.parse::<f64>().map_err(|_| ())
}

fn check_nan(value: f64) -> Result<f64, ()> {
    if value.is_nan() {
        Err(())
    } else {
        Ok(value)
    }
}

fn check_zero(value: f64) -> Result<f64, ()> {
    if value == 0.0 {
        Err(())
    } else {
        Ok(value)
    }
}

fn argument_error(field: &str) -> impl Fn(()) -> f64 + '_ {
    move |_| {
        eprintln!(
            "seq: Invalid {}.\nTry 'seq --help' for more information.",
            field
        );
        process::exit(1);
    }
}

const DEFAULT_FIRST: &str = "1";
const DEFAULT_INCREMENT: &str = "1";

fn main() {
    let yaml = load_yaml!("seq.yml");
    let matches = App::from_yaml(yaml)
        .setting(AppSettings::AllowNegativeNumbers)
        .get_matches();

    let (raw_first, raw_increment, raw_last) = match (
        matches.value_of("FIRST"),
        matches.value_of("SECOND"),
        matches.value_of("THIRD"),
    ) {
        (Some(last), None, None) => (DEFAULT_FIRST, DEFAULT_INCREMENT, last),
        (Some(first), Some(last), None) => (first, DEFAULT_INCREMENT, last),
        (Some(first), Some(increment), Some(last)) => (first, increment, last),
        _ => {
            eprintln!("seq: Missing operands.\nTry 'seq --help' for more information.");
            process::exit(1);
        }
    };

    let first = parse_number(raw_first)
        .and_then(check_nan)
        .unwrap_or_else(argument_error("FIRST"));

    let last = parse_number(raw_last)
        .and_then(check_nan)
        .unwrap_or_else(argument_error("LAST"));

    let increment = parse_number(raw_increment)
        .and_then(check_nan)
        .and_then(check_zero)
        .unwrap_or_else(argument_error("INCREMENT"));

    let iter = Seq {
        current: first,
        stop: last,
        step: increment,
    };

    for index in iter {
        println!("{}", index);
    }
}
