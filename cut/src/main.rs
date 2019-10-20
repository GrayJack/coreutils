use clap::{load_yaml, App, ArgMatches};
use std::{
    cmp::min,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    num::ParseIntError,
    result, string,
    usize::{MAX, MIN},
};

fn main() {
    let yaml = load_yaml!("cut.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let filenames: Vec<_> = match matches.values_of("FILE") {
        Some(files) => files.collect(),
        None => vec!["-"],
    };

    let line_terminator = if matches.is_present("zero-terminated") { '\0' } else { '\n' } as u8;
    let complement = matches.is_present("complement");
    let options = Options { line_terminator, complement };

    let result = make_cutter(&matches, &options).and_then(|cutter| {
        filenames
            .iter()
            .map(|filename| cutter.process_file(&filename, &options))
            .collect::<Result<Vec<_>>>()
    });

    if let Err(err) = result {
        eprintln!("cut: {}", err);
    }
}

struct Options {
    line_terminator: u8,
    complement:      bool,
}

#[derive(PartialEq, Debug)]
enum Error {
    SyntaxError(String),
    RangeError(String),
    ParseError(String),
    IOError(String),
    InternalError(String),
}

impl From<ParseIntError> for Error {
    fn from(_err: ParseIntError) -> Error { Error::ParseError(format!("not an integer")) }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error { Error::IOError(format!("{}", err)) }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error { Error::ParseError(format!("{}", err)) }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SyntaxError(ref msg) => write!(f, "syntax error: {}", msg),
            Error::RangeError(ref msg) => write!(f, "range error: {}", msg),
            Error::ParseError(ref msg) => write!(f, "parse error: {}", msg),
            Error::IOError(ref msg) => write!(f, "I/O error: {}", msg),
            Error::InternalError(ref msg) => write!(f, "internal error: {}", msg),
        }
    }
}

type Result<T> = result::Result<T, Error>;

/// Range of the form [start, one-after-end).
#[derive(Debug, PartialEq, Clone, Copy)]
struct Range(usize, usize);

impl Range {
    /// Parse a range into optional beginning and optional end.
    ///
    /// Accepted formats are:
    /// - <number>
    /// - "-" <number>
    /// - <number> "-"
    /// - <number> "-" <number>
    /// # Errors
    fn from_string(string: &str) -> Result<Range> {
        let v: Vec<&str> = string.split('-').collect();
        if string.len() == 0 || v.len() < 1 || v.len() > 2 {
            return Err(Error::SyntaxError(format!("invalid byte or character range")));
        }

        // An interval with no endpoints at all should give an error.
        if v.len() == 2 && v[0].len() == 0 && v[1].len() == 0 {
            return Err(Error::RangeError(format!("invalid range with no endpoint")));
        }

        let lower = if v[0].len() == 0 { MIN } else { v[0].parse::<usize>()? - 1 };
        let upper = if v.len() == 1 {
            lower + 1
        } else if v[1].len() == 0 {
            MAX
        } else {
            v[1].parse::<usize>()?
        };

        if lower >= upper {
            return Err(Error::RangeError(format!(
                "invalid range {} ({} >= {})",
                string,
                lower + 1,
                upper
            )));
        }

        Ok(Range(lower, upper))
    }
}

/// A set of ranges.
#[derive(PartialEq, Debug)]
struct RangeSet {
    pub points: Vec<Range>,
}

impl RangeSet {
    fn from_string(string: &str) -> Result<RangeSet> {
        // Split the string at commas and parse the pieces as ranges.
        let ranges =
            string.split(',').map(|rng| Range::from_string(rng)).collect::<Result<Vec<Range>>>()?;
        RangeSet::from_vec(ranges)
    }

    fn from_vec(mut ranges: Vec<Range>) -> Result<RangeSet> {
        // Sort the ranges on the start of the range. This will place
        // all ranges in correct order in the vector for the merging
        // below.
        ranges.sort_unstable_by_key(|rng| rng.0);

        // Iterate over the ranges and merge ranges if there are
        // any overlaps.
        let mut current: Option<Range> = None;
        let mut points = Vec::new();
        for range in &ranges {
            if let Some(rng) = current {
                if range.0 <= rng.1 {
                    current = Some(Range(rng.0, range.1));
                } else {
                    points.push(rng);
                    current = Some(*range);
                }
            } else {
                current = Some(*range);
            }
        }

        if let Some(rng) = current {
            points.push(rng);
        }
        Ok(RangeSet { points })
    }

    // In-place complement a range set.
    fn complement(&mut self) {
        let mut points = Vec::new();
        let mut carry = 0;
        for range in &self.points {
            if range.0 > carry {
                points.push(Range(carry, range.0));
            }
            carry = range.1;
        }
        if carry < MAX {
            points.push(Range(carry, MAX));
        }
        self.points = points;
    }
}

// Trait that is used to implement line cutting traits.
trait Cutter {
    fn process_line(&self, line: Vec<u8>) -> Result<()>;

    // Process an entire file. The special file name "-" will be
    // reading from standard input.
    fn process_file(&self, filename: &str, options: &Options) -> Result<()> {
        let mut reader: Box<dyn io::Read> =
            if filename == "-" { Box::new(io::stdin()) } else { Box::new(File::open(filename)?) };
        self.process_input(&mut reader, options)
    }

    // Process input from an already opened reader.
    fn process_input(&self, reader: &mut Box<dyn io::Read>, options: &Options) -> Result<()> {
        let mut reader = BufReader::new(reader);
        loop {
            let mut line = Vec::new();
            match reader.read_until(options.line_terminator, &mut line) {
                Ok(count) if count > 0 => self.process_line(line)?,
                Ok(_) => return Ok(()),
                Err(err) => return Err(Error::IOError(format!("I/O error: {}", err))),
            }
        }
    }
}

// A byte cutter that will cut out bytes by position in the line.
struct Bytes {
    range_set: RangeSet,
}

impl Bytes {
    fn new(range_set: RangeSet, _matches: &ArgMatches) -> Result<Bytes> { Ok(Bytes { range_set }) }
}

impl Cutter for Bytes {
    fn process_line(&self, bytes: Vec<u8>) -> Result<()> {
        // If line is shorter than range give, only print the parts of
        // the line that are in range.
        for range in &self.range_set.points {
            if bytes.len() > range.0 {
                io::stdout().write_all(&bytes[range.0..min(bytes.len(), range.1)])?;
            }
        }
        io::stdout().write_all(b"\n")?;
        Ok(())
    }
}

// A character cutter that will cut out character by position in the
// line.
struct Chars {
    range_set: RangeSet,
}

impl Chars {
    fn new(range_set: RangeSet, _matches: &ArgMatches) -> Result<Chars> { Ok(Chars { range_set }) }
}

impl Cutter for Chars {
    fn process_line(&self, bytes: Vec<u8>) -> Result<()> {
        let line: String = String::from_utf8(bytes)?;
        let pieces: Vec<&str> = self
            .range_set
            .points
            .iter()
            .map(|range| {
                // If line is shorter than range give, only print the
                // parts of the line that are in range.
                if line.len() > range.0 { &line[range.0..min(line.len(), range.1)] } else { "" }
            })
            .collect();
        println!("{}", pieces.join(""));
        Ok(())
    }
}

// A field cutter that will cut out delimited fields of the line.
struct Fields {
    range_set: RangeSet,
    only_delimited: bool,
    input_delimiter: String,
    output_delimiter: String,
}

impl Fields {
    fn new(range_set: RangeSet, matches: &ArgMatches) -> Result<Fields> {
        let idelim = matches.value_of("input-delimiter").unwrap_or("\t");
        if idelim.len() != 1 {
            return Err(Error::SyntaxError(format!("single character for delimiter")));
        }

        let odelim = matches.value_of("output-delimiter").unwrap_or(idelim);
        if odelim.len() != 1 {
            return Err(Error::SyntaxError(format!("single character for delimiter")));
        }

        Ok(Fields {
            range_set,
            only_delimited: matches.is_present("only-delimited"),
            input_delimiter: idelim.to_string(),
            output_delimiter: odelim.to_string(),
        })
    }
}

impl Cutter for Fields {
    fn process_line(&self, bytes: Vec<u8>) -> Result<()> {
        let line: String = String::from_utf8(bytes)?;
        let fields: Vec<&str> = line.split(&self.input_delimiter).collect();
        if !self.only_delimited || fields.len() > 1 {
            let pieces: Vec<_> = self
                .range_set
                .points
                .iter()
                .map(|range| {
                    // If there are fewer fields than what the range
                    // denotes, we print those fields that are in the
                    // range.
                    if fields.len() > range.0 {
                        fields[range.0..min(fields.len(), range.1)].join(&self.output_delimiter)
                    } else {
                        "".to_string()
                    }
                })
                .collect();
            println!("{}", pieces.join(&self.output_delimiter));
        }
        Ok(())
    }
}

// Factory function to create a cutter from command-line arguments.
fn make_cutter(matches: &ArgMatches, options: &Options) -> Result<Box<dyn Cutter>> {
    if let Some(rng) = matches.value_of("bytes") {
        let mut range_set = RangeSet::from_string(rng)?;
        if options.complement {
            range_set.complement();
        }
        let cutter = Bytes::new(range_set, matches)?;
        Ok(Box::new(cutter))
    } else if let Some(rng) = matches.value_of("chars") {
        let mut range_set = RangeSet::from_string(rng)?;
        if options.complement {
            range_set.complement();
        }
        let cutter = Chars::new(range_set, matches)?;
        Ok(Box::new(cutter))
    } else if let Some(rng) = matches.value_of("fields") {
        let mut range_set = RangeSet::from_string(rng)?;
        if options.complement {
            range_set.complement();
        }
        let cutter = Fields::new(range_set, matches)?;
        Ok(Box::new(cutter))
    } else {
        Err(Error::InternalError(format!("not possible to select cutter")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::usize::{MAX, MIN};

    // Macro to assert that an expression matches a pattern.
    macro_rules! assert_matches {
        ($xpr:expr, $pat:pat) => {
            match $xpr {
                $pat => true,
                ref xpr => {
                    panic!("assert_matches: '{:?}' doesn't match '{}'", xpr, stringify!($pat))
                },
            }
        };
    }

    #[test]
    fn range_from_string() {
        assert_eq!(Range::from_string("2"), Ok(Range(1, 2)));
        assert_eq!(Range::from_string("-2"), Ok(Range(MIN, 2)));
        assert_eq!(Range::from_string("2-"), Ok(Range(1, MAX)));
        assert_eq!(Range::from_string("2-5"), Ok(Range(1, 5)));

        assert_matches!(Range::from_string(""), Err(Error::SyntaxError(_)));
        assert_matches!(Range::from_string("5-2"), Err(Error::RangeError(_)));
        assert_matches!(Range::from_string("foo"), Err(Error::ParseError(_)));
        assert_matches!(Range::from_string("2-0x12"), Err(Error::ParseError(_)));
        assert_matches!(Range::from_string("-"), Err(Error::RangeError(_)));
    }

    #[test]
    fn rangeset_from_string() {
        assert_eq!(RangeSet::from_string("2"), Ok(RangeSet { points: vec![Range(1, 2)] }));
        assert_eq!(RangeSet::from_string("-2"), Ok(RangeSet { points: vec![Range(MIN, 2)] }));
        assert_eq!(RangeSet::from_string("2,3"), Ok(RangeSet { points: vec![Range(1, 3)] }));
        assert_eq!(RangeSet::from_string("2-3"), Ok(RangeSet { points: vec![Range(1, 3)] }));
        assert_eq!(
            RangeSet::from_string("2-3,3-5,4-6"),
            Ok(RangeSet { points: vec![Range(1, 6)] })
        );
        assert_eq!(
            RangeSet::from_string("4-6,3-5,2-3"),
            Ok(RangeSet { points: vec![Range(1, 6)] })
        );
        assert_eq!(
            RangeSet::from_string("2,5-10"),
            Ok(RangeSet { points: vec![Range(1, 2), Range(4, 10)] })
        );
        assert_eq!(
            RangeSet::from_string("2,5-"),
            RangeSet::from_vec(vec![Range(1, 2), Range(4, MAX)])
        );
        assert_eq!(
            RangeSet::from_string("-2,5-"),
            Ok(RangeSet { points: vec![Range(MIN, 2), Range(4, MAX)] })
        );
    }

    fn complement_rangeset_helper(ranges: Vec<Range>, expected: Vec<Range>) {
        let mut range_set = RangeSet::from_vec(ranges).unwrap();
        range_set.complement();
        assert_eq!(range_set, RangeSet::from_vec(expected).unwrap());
    }

    #[test]
    fn completment_rangeset() {
        complement_rangeset_helper(vec![Range(MIN, MAX)], vec![]);
        complement_rangeset_helper(vec![Range(MIN, 5)], vec![Range(5, MAX)]);
        complement_rangeset_helper(vec![Range(5, MAX)], vec![Range(MIN, 5)]);
        complement_rangeset_helper(vec![Range(1, 5)], vec![Range(MIN, 1), Range(5, MAX)]);
        complement_rangeset_helper(vec![Range(1, 5), Range(8, 12)], vec![
            Range(MIN, 1),
            Range(5, 8),
            Range(12, MAX),
        ]);
        complement_rangeset_helper(vec![Range(MIN, 5), Range(8, 12)], vec![
            Range(5, 8),
            Range(12, MAX),
        ]);
        complement_rangeset_helper(vec![Range(5, 8), Range(12, MAX)], vec![
            Range(0, 5),
            Range(8, 12),
        ]);
    }
}
