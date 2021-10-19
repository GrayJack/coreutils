mod args;
mod cli;
mod float;

use std::{
    cell::RefCell,
    fmt::{self, Display, Write},
    process,
};

use args::Args;

fn main() {
    let matches = cli::create_app().get_matches();
    let args = match Args::parse(&matches) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("seq: {}", error);
            process::exit(1);
        },
    };

    let seq = Seq::new(&args);
    print!("{}", seq);
}


#[derive(Debug)]
struct Seq<'a> {
    first: f64,
    increment: f64,
    last: f64,
    decimals: usize,
    padding: Option<usize>,
    separator: &'a str,
    terminator: &'a str,
    buffer: RefCell<String>,
}


impl<'a> Seq<'a> {
    fn new(args: &Args<'a>) -> Self {
        Self {
            first: args.first,
            increment: args.increment,
            last: args.last,
            decimals: args.decimals,
            padding: args.padding,
            separator: args.separator,
            terminator: args.terminator,
            buffer: Default::default(),
        }
    }

    fn is_finished(&self, value: f64) -> bool {
        if self.increment > 0.0 { value > self.last } else { value < self.last }
    }

    fn format<W: Write>(&self, mut writer: W, value: f64) -> fmt::Result {
        let mut buffer = self.buffer.borrow_mut();

        buffer.clear();
        write!(buffer, "{:.*}", self.decimals, value)?;

        let digits = float::count_integer_digits(&buffer);

        if let Some(padding) = self.padding {
            if padding > digits {
                buffer.clear();

                let value = if value < 0.0 {
                    buffer.push('-');
                    value.abs()
                } else {
                    value
                };

                buffer.extend(std::iter::repeat('0').take(padding - digits));

                write!(buffer, "{:.*}", self.decimals, value)?;
            }
        }

        writer.write_str(&buffer)
    }
}


impl<'a> Display for Seq<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut current = self.first;

        if self.is_finished(current) {
            return Ok(());
        }

        let mut buffer = String::new();
        self.format(&mut buffer, current)?;
        f.write_str(&buffer)?;

        current += self.increment;

        while !self.is_finished(current) {
            buffer.clear();
            self.format(&mut buffer, current)?;
            write!(f, "{}{}", self.separator, buffer)?;

            current += self.increment;
        }

        // If the number past self.last prints equal to self.last, and prints differently
        // from the previous number, then we should print it. This avoids problems with
        // rounding.

        let mut last = String::with_capacity(buffer.len());
        self.format(&mut last, self.last)?;

        if last != buffer {
            // Last was not already printed.
            buffer.clear();
            self.format(&mut buffer, current)?;

            if buffer == last {
                // The final number prints the same as last.
                write!(f, "{}{}", self.separator, buffer)?;
            }
        }

        f.write_str(self.terminator)
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    fn seq(
        first: f64, increment: f64, last: f64, decimals: usize, padding: Option<usize>,
    ) -> Seq<'static> {
        Seq {
            first,
            increment,
            last,
            decimals,
            padding,
            separator: ",",
            terminator: ";",
            buffer: Default::default(),
        }
    }


    #[test]
    fn should_generate_sequence() {
        assert_eq!(seq(1.0, 1.0, 1.0, 0, None).to_string(), "1;");

        assert_eq!(seq(2.0, 1.0, 2.0, 0, None).to_string(), "2;");

        assert_eq!(seq(1.0, 1.0, 3.0, 0, None).to_string(), "1,2,3;");

        assert_eq!(seq(1.0, 1.0, 3.0, 1, None).to_string(), "1.0,2.0,3.0;");

        assert_eq!(seq(1.0, 0.2, 2.0, 1, None).to_string(), "1.0,1.2,1.4,1.6,1.8,2.0;");

        assert_eq!(
            // #133
            seq(0.1, 0.01, 0.2, 2, None).to_string(),
            "0.10,0.11,0.12,0.13,0.14,0.15,0.16,0.17,0.18,0.19,0.20;"
        );

        assert_eq!(seq(1.0, 0.2, 2.0, 3, None).to_string(), "1.000,1.200,1.400,1.600,1.800,2.000;");

        assert_eq!(seq(-2.0, 1.0, 2.0, 0, None).to_string(), "-2,-1,0,1,2;");

        assert_eq!(seq(-1.0, -5.0, -15.0, 0, Some(4)).to_string(), "-001,-006,-011;");

        assert_eq!(seq(1.0, 5.0, 16.0, 0, Some(2)).to_string(), "01,06,11,16;");
    }
}
