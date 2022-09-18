use std::fmt::{self, Display};

use super::float;

#[derive(Debug)]
pub(crate) enum Error<'a> {
    InvalidIncrement(&'a str),
    InvalidFloat(&'a str),
    MissingOperand,
    TrailingOperand(&'a str),
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidIncrement(increment) => write!(f, "invalid increment '{}'", increment),
            Self::InvalidFloat(input) => write!(f, "invalid floating point argument '{}'", input),
            Self::MissingOperand => write!(f, "missing operand"),
            Self::TrailingOperand(operand) => write!(f, "extra operand '{}'", operand),
        }
    }
}

impl<'a> std::error::Error for Error<'a> {}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Args<'a> {
    pub first: f64,
    pub increment: f64,
    pub last: f64,
    pub decimals: usize,
    pub padding: Option<usize>,
    pub separator: &'a str,
    pub terminator: &'a str,
}

impl<'a> Args<'a> {
    pub fn parse(matches: &'a clap::ArgMatches) -> Result<Self, Error<'a>> {
        let operands: Vec<&str> =
            matches.values_of("FIRST INCREMENT LAST").ok_or(Error::MissingOperand)?.collect();

        let (first, increment, last) = Self::parse_operands(&operands)?;

        let decimals = operands
            .iter()
            .copied()
            .map(float::count_decimal_digits)
            .max()
            .ok_or(Error::MissingOperand)?;

        let padding = if matches.is_present("equal-width") {
            let padding = operands
                .iter()
                .copied()
                .map(float::count_integer_digits)
                .max()
                .ok_or(Error::MissingOperand)?;

            Some(padding)
        } else {
            None
        };

        Ok(Self {
            first,
            increment,
            last,

            decimals,
            padding,

            separator: matches
                .value_of("separator")
                .expect("missing default argument for separator"),

            terminator: matches
                .value_of("terminator")
                .expect("missing default argument for terminator"),
        })
    }

    fn parse_operands(operands: &[&'a str]) -> Result<(f64, f64, f64), Error<'a>> {
        match operands {
            [] => Err(Error::MissingOperand),

            [last] => {
                let last = Self::parse_float(last)?;
                Ok((1.0, 1.0, last))
            },

            [first, last] => {
                let first = Self::parse_float(first)?;
                let last = Self::parse_float(last)?;
                Ok((first, 1.0, last))
            },

            [first, inc, last] => {
                let first = Self::parse_float(first)?;
                let increment = Self::parse_float(inc)?;
                let last = Self::parse_float(last)?;

                if increment == 0.0 {
                    Err(Error::InvalidIncrement(inc))
                } else {
                    Ok((first, increment, last))
                }
            },

            [_, _, _, trailing, ..] => Err(Error::TrailingOperand(trailing)),
        }
    }

    fn parse_float(arg: &str) -> Result<f64, Error> {
        arg.parse().map_err(|_| Error::InvalidFloat(arg))
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::*;
    use crate::cli;


    fn test_input<I, T>(iterator: I, expected: &Args)
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = cli::create_app().get_matches_from(iterator);
        let args = Args::parse(&matches).expect("failed to parse matches");

        assert_eq!(&args, expected);
    }


    #[test]
    fn test_parsing_input() {
        test_input(["seq", "1", "1", "4"], &Args {
            first: 1.0,
            increment: 1.0,
            last: 4.0,
            decimals: 0,
            padding: None,
            separator: "\n",
            terminator: "\n",
        });

        test_input(["seq", "2", "10"], &Args {
            first: 2.0,
            increment: 1.0,
            last: 10.0,
            decimals: 0,
            padding: None,
            separator: "\n",
            terminator: "\n",
        });

        test_input(["seq", "3"], &Args {
            first: 1.0,
            increment: 1.0,
            last: 3.0,
            decimals: 0,
            padding: None,
            separator: "\n",
            terminator: "\n",
        });
    }
}
