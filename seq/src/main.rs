use std::process;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    if let Some(values) = matches.values_of("FIRST INCREMENT LAST") {
        let args: Vec<&str> = values.collect();
        if args.len() > 3 {
            eprintln!("seq: extra operand '{}'\n Try 'seq --help' for more information.", args[3]);
            process::exit(1);
        }
        let seperator = matches.value_of("SEPERATOR").map(String::from).unwrap();
        let decimals = max_decimal_digits(&args);
        let padding = if matches.is_present("WIDTH") { Some(max_digits(&args)) } else { None };
        let (first, inc, last) = find_operands(&args);
        let valid_range = (first < last && inc > 0.0) || (first > last && inc < 0.0);
        if valid_range {
            let seq = Seq::new(first, inc, last, decimals, seperator, padding);
            for val in seq.into_iter() {
                print!("{}", val);
            }
            println!();
        }
    } else {
        eprintln!("seq: missing operand\n Try 'seq --help' for more information.");
        process::exit(1);
    }
}

fn find_operands(args: &[&str]) -> (f64, f64, f64) {
    match args.len() {
        1 => (1.0, 1.0, parse_float(args[0])),
        2 => (parse_float(args[0]), 1.0, parse_float(args[1])),
        _ => (parse_float(args[0]), parse_float(args[1]), parse_float(args[2])),
    }
}

fn parse_float(s: &str) -> f64 {
    s.parse::<f64>().unwrap_or_else(|_| {
        eprintln!("seq: invalid floating point argument: {}", s);
        process::exit(1);
    })
}

struct Seq {
    first:     f64,
    inc:       f64,
    last:      f64,
    decimals:  usize,
    seperator: String,
    padding:   Option<usize>,
}

impl Seq {
    fn new(
        first: f64, inc: f64, last: f64, decimals: usize, seperator: String, padding: Option<usize>,
    ) -> Seq {
        Seq { first, inc, last, decimals, seperator, padding }
    }

    fn is_complete(&self, value: f64) -> bool {
        self.inc > 0.0 && value > self.last || self.inc < 0.0 && value < self.last
    }
}

fn max_decimal_digits(args: &[&str]) -> usize {
    // args will never be empty and all elements are already validated as f64
    args.iter()
        .map(|v| v.len() - v.find('.').map(|pos| pos + 1).unwrap_or_else(|| v.len()))
        .max()
        .unwrap()
}

fn max_digits(args: &[&str]) -> usize {
    // args will never be empty and each element is already validated as f64
    args.iter().map(|v| v.find('.').unwrap_or_else(|| v.len())).max().unwrap()
}

impl IntoIterator for Seq {
    type IntoIter = SeqIterator;
    type Item = String;

    fn into_iter(self) -> Self::IntoIter {
        let current = self.first;
        SeqIterator { seq: self, current }
    }
}

struct SeqIterator {
    seq:     Seq,
    current: f64,
}

impl Iterator for SeqIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.seq.is_complete(self.current) {
            None
        } else {
            let value = format!("{:.*}", self.seq.decimals, self.current);
            let digits = value.find('.').unwrap_or_else(|| value.len());

            let mut value = match self.seq.padding {
                Some(width) if width > digits => {
                    let mut padded = String::with_capacity(value.len() + (width - digits));

                    if value.starts_with('-') {
                        padded.push_str("-");
                        padded.push_str(&"0".repeat(width - digits));
                        padded.push_str(&value[1..]);
                    } else {
                        padded.push_str(&"0".repeat(width - digits));
                        padded.push_str(&value);
                    }

                    padded
                },
                _ => value,
            };

            if !self.seq.is_complete(self.current + self.seq.inc) {
                value.push_str(&self.seq.seperator);
            }
            self.current += self.seq.inc;
            Some(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_find_max_decimal_digits() {
        assert_eq!(max_decimal_digits(&["1.22", "1.3", "1"]), 2);
        assert_eq!(max_decimal_digits(&["1", "1.3", "1.1"]), 1);
        assert_eq!(max_decimal_digits(&["1"]), 0);
    }

    #[test]
    fn should_find_max_digits() {
        assert_eq!(max_digits(&["1.22", "12.3", "1"]), 2);
        assert_eq!(max_digits(&["1.22", "12.3", "123"]), 3);
        assert_eq!(max_digits(&["1.22", "-152.3", "123"]), 4);
    }

    #[test]
    fn should_find_operands() {
        assert_eq!(find_operands(&["2", "3", "10"]), (2.0, 3.0, 10.0));
        assert_eq!(find_operands(&["2", "10"]), (2.0, 1.0, 10.0));
        assert_eq!(find_operands(&["3"]), (1.0, 1.0, 3.0));
    }

    fn to_string(xs: Vec<&str>) -> Vec<String> {
        xs.into_iter().map(|v: &str| v.to_owned()).collect::<Vec<String>>()
    }

    #[test]
    fn should_generate_sequence() {
        assert_eq!(
            Seq::new(1.0, 1.0, 3.0, 0, "".to_owned(), None).into_iter().collect::<Vec<String>>(),
            to_string(vec!["1", "2", "3"])
        );

        assert_eq!(
            Seq::new(1.0, 1.0, 3.0, 0, ",".to_owned(), None).into_iter().collect::<Vec<String>>(),
            to_string(vec!["1,", "2,", "3"])
        );

        assert_eq!(
            Seq::new(1.0, 1.0, 3.0, 1, "".to_owned(), None).into_iter().collect::<Vec<String>>(),
            to_string(vec!["1.0", "2.0", "3.0"])
        );

        assert_eq!(
            Seq::new(1.0, 0.2, 2.0, 1, "".to_owned(), None).into_iter().collect::<Vec<String>>(),
            to_string(vec!["1.0", "1.2", "1.4", "1.6", "1.8", "2.0"])
        );

        assert_eq!(
            Seq::new(1.0, 0.2, 2.0, 3, "".to_owned(), None).into_iter().collect::<Vec<String>>(),
            to_string(vec!["1.000", "1.200", "1.400", "1.600", "1.800", "2.000"])
        );

        assert_eq!(
            Seq::new(-2.0, 1.0, 2.0, 0, "".to_owned(), None).into_iter().collect::<Vec<String>>(),
            to_string(vec!["-2", "-1", "0", "1", "2"])
        );

        assert_eq!(
            Seq::new(-1.0, -5.0, -15.0, 0, "".to_owned(), Some(4))
                .into_iter()
                .collect::<Vec<String>>(),
            to_string(vec!["-001", "-006", "-011"])
        );

        assert_eq!(
            Seq::new(1.0, 1.0, 3.0, 0, "".to_owned(), None).into_iter().collect::<Vec<String>>(),
            to_string(vec!["1", "2", "3"])
        );

        assert_eq!(
            Seq::new(1.0, 5.0, 16.0, 0, "".to_owned(), Some(2))
                .into_iter()
                .collect::<Vec<String>>(),
            to_string(vec!["01", "06", "11", "16"])
        );
    }
}
