use clap::{load_yaml, App, ArgMatches};
use std::{
    cmp::min,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader},
    num::ParseIntError,
    result,
};

fn main() {
    let yaml = load_yaml!("cut.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let filenames: Vec<_> = match matches.values_of("FILE") {
        Some(files) => files.collect(),
        None => vec!["-"],
    };

    let result = make_cutter(&matches).and_then(|cutter| {
        filenames.iter().map(|filename| cutter.process_file(&filename)).collect::<Result<Vec<_>>>()
    });

    if let Err(err) = result {
        eprintln!("cut: {}", err);
    }
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

        let lower = if v[0].len() == 0 { std::usize::MIN } else { v[0].parse::<usize>()? - 1 };
        let upper = if v.len() == 1 {
            lower + 1
        } else if v[1].len() == 0 {
            std::usize::MAX
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
        let mut ranges =
            string.split(',').map(|rng| Range::from_string(rng)).collect::<Result<Vec<Range>>>()?;

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
}

// Trait that is used to implement line cutting traits.
trait Cutter {
    fn process_line(&self, line: &str);

    // Process an entire file. The special file name "-" will be
    // reading from standard input.
    fn process_file(&self, filename: &str) -> Result<()> {
        let mut reader: Box<dyn io::Read> =
            if filename == "-" { Box::new(io::stdin()) } else { Box::new(File::open(filename)?) };
        self.process_input(&mut reader)
    }

    // Process input from an already opened reader.
    fn process_input(&self, reader: &mut Box<dyn io::Read>) -> Result<()> {
        let reader = BufReader::new(reader);
        for line in reader.lines() {
            match line {
                Ok(ref line) => self.process_line(line),
                Err(err) => return Err(Error::IOError(format!("I/O error: {}", err))),
            }
        }
        Ok(())
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
    fn process_line(&self, line: &str) {
        // If line is shorter than range give, only print the parts of
        // the line that are in range.
        for range in &self.range_set.points {
            if line.len() > range.0 {
                print!("{}", &line[range.0..min(line.len(), range.1)]);
            }
        }
        print!("\n");
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
    fn process_line(&self, line: &str) {
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
    fn process_line(&self, line: &str) {
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
    }
}

// Factory function to create a cutter from command-line arguments.
fn make_cutter(matches: &ArgMatches) -> Result<Box<dyn Cutter>> {
    let line_delimiter = if matches.is_present("zero-terminated") { '\0' } else { '\n' };
    if let Some(rng) = matches.value_of("bytes") {
        let range_set = RangeSet::from_string(rng)?;
        let cutter = Bytes::new(range_set, matches)?;
        Ok(Box::new(cutter))
    } else if let Some(rng) = matches.value_of("chars") {
        let range_set = RangeSet::from_string(rng)?;
        let cutter = Chars::new(range_set, matches)?;
        Ok(Box::new(cutter))
    } else if let Some(rng) = matches.value_of("fields") {
        let range_set = RangeSet::from_string(rng)?;
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
            Ok(RangeSet { points: vec![Range(1, 2), Range(4, MAX)] })
        );
        assert_eq!(
            RangeSet::from_string("-2,5-"),
            Ok(RangeSet { points: vec![Range(MIN, 2), Range(4, MAX)] })
        );
    }
}
