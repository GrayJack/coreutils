#![allow(clippy::suspicious_arithmetic_impl)]
use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, AddAssign},
    process,
    str::FromStr
};

use clap::{load_yaml, App, AppSettings};
use rust_decimal::Decimal;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Number {
    NegInf,
    PosInf,
    NaN,
    Num(Decimal)
}

use std::cmp::Ordering::*;
use Number::*;

impl Add for Number {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Num(left), Num(right)) => match left.checked_add(right) {
                Some(next_dec) => Num(next_dec),
                None => {
                    if left.is_sign_positive() && right.is_sign_positive() {
                        PosInf
                    } else if left.is_sign_negative() && right.is_sign_negative() {
                        NegInf
                    } else {
                        NaN
                    }
                },
            },
            (NaN, _) | (_, NaN) | (PosInf, NegInf) | (NegInf, PosInf) => NaN,
            (PosInf, PosInf) | (PosInf, Num(_)) | (Num(_), PosInf) => PosInf,
            (NegInf, NegInf) | (NegInf, Num(_)) | (Num(_), NegInf) => NegInf
        }
    }
}

impl AddAssign for Number {
    fn add_assign(&mut self, other: Self) { *self = *self + other; }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Num(left_dec), Num(right_dec)) => Some(left_dec.cmp(right_dec)),
            (NaN, NaN) => Some(Equal),
            (NaN, _) | (_, NaN) => None,
            (PosInf, PosInf) | (NegInf, NegInf) => Some(Equal),
            (NegInf, PosInf) | (Num(_), PosInf) | (NegInf, Num(_)) => Some(Less),
            (PosInf, NegInf) | (PosInf, Num(_)) | (Num(_), NegInf) => Some(Greater)
        }
    }
}

impl From<f64> for Number {
    fn from(number: f64) -> Self {
        match number {
            value if value.is_infinite() => {
                if value.is_sign_positive() {
                    PosInf
                } else {
                    NegInf
                }
            },
            value if value.is_finite() => Num(Decimal::from_str(&value.to_string()).unwrap()),
            _ => NaN
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            PosInf => write!(f, "inf"),
            NegInf => write!(f, "-inf"),
            NaN => write!(f, "nan"),
            Num(value) => write!(f, "{}", value)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Seq {
    current: Number,
    step:    Number,
    stop:    Number
}

impl Iterator for Seq {
    type Item = Number;

    fn next(&mut self) -> Option<Self::Item> {
        match self.step {
            step @ PosInf | step @ NegInf => Some(step + self.current),
            Num(step_dec) => {
                if (step_dec.is_sign_positive() && self.current <= self.stop)
                    || (step_dec.is_sign_negative() && self.current >= self.stop)
                {
                    let result = self.current;

                    self.current += self.step;

                    Some(result)
                } else {
                    None
                }
            },
            NaN => Some(NaN)
        }
    }
}

fn parse_number(text: &str) -> Result<f64, ()> { text.parse::<f64>().map_err(|_| ()) }

fn check_nan(value: f64) -> Result<f64, ()> { if value.is_nan() { Err(()) } else { Ok(value) } }

fn check_zero(value: f64) -> Result<f64, ()> { if value == 0.0 { Err(()) } else { Ok(value) } }

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
const DEFAULT_SEPARATOR: &str = "\n";

fn main() {
    let yaml = load_yaml!("seq.yml");
    let matches = App::from_yaml(yaml)
        .setting(AppSettings::AllowNegativeNumbers)
        .get_matches();

    let (raw_first, raw_increment, raw_last) = match (
        matches.value_of("FIRST"),
        matches.value_of("SECOND"),
        matches.value_of("THIRD")
    ) {
        (Some(last), None, None) => (DEFAULT_FIRST, DEFAULT_INCREMENT, last),
        (Some(first), Some(last), None) => (first, DEFAULT_INCREMENT, last),
        (Some(first), Some(increment), Some(last)) => (first, increment, last),
        _ => {
            eprintln!("seq: Missing operands.\nTry 'seq --help' for more information.");
            process::exit(1);
        }
    };

    let separator = matches.value_of("SEPARATOR").unwrap_or(DEFAULT_SEPARATOR);

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

    let mut iter = (Seq {
        current: Number::from(first),
        stop:    Number::from(last),
        step:    Number::from(increment)
    })
    .peekable();

    while let Some(index) = iter.next() {
        if *&iter.peek().is_some() {
            print!("{}{}", index, separator);
        } else {
            println!("{}", index);
        }
    }
}
