use std::{
    cmp::min,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    num::ParseIntError,
    process, result, string,
};

use clap::ArgMatches;

#[cfg(test)]
mod tests;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

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
            .map(|filename| cutter.process_file(filename, &options))
            .collect::<Result<Vec<_>>>()
    });

    if let Err(err) = result {
        eprintln!("cut: {}", err);
        process::exit(1);
    }
}

struct Options {
    line_terminator: u8,
    complement: bool,
}

#[derive(PartialEq, Debug)]
struct Error(String, i32);

impl From<ParseIntError> for Error {
    fn from(_err: ParseIntError) -> Self {
        Error("not an integer".to_string(), 2)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error(format!("{}", err), 1)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Self {
        Error(format!("{}", err), 1)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
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
    fn from_string(string: &str) -> Result<Self> {
        let v: Vec<&str> = string.split('-').collect();
        if string.is_empty() || v.is_empty() || v.len() > 2 {
            return Err(Error("invalid byte or character range".to_string(), 2));
        }

        // An interval with no endpoints at all should give an error.
        if v.len() == 2 && v[0].is_empty() && v[1].is_empty() {
            return Err(Error("invalid range with no endpoint".to_string(), 2));
        }

        let lower = if v[0].is_empty() { usize::min_value() } else { v[0].parse::<usize>()? - 1 };
        let upper = if v.len() == 1 {
            lower + 1
        } else if v[1].is_empty() {
            usize::max_value()
        } else {
            v[1].parse::<usize>()?
        };

        if lower >= upper {
            return Err(Error(format!("invalid range {} ({} >= {})", string, lower + 1, upper), 2));
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
    fn from_string(string: &str) -> Result<Self> {
        // Split the string at commas and parse the pieces as ranges.
        let ranges = string.split(',').map(Range::from_string).collect::<Result<Vec<Range>>>()?;
        Ok(Self::from_vec(ranges))
    }

    fn from_vec(mut ranges: Vec<Range>) -> Self {
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
        RangeSet { points }
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
        if carry < usize::max_value() {
            points.push(Range(carry, usize::max_value()));
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
    fn process_input(&self, reader: &mut dyn io::Read, options: &Options) -> Result<()> {
        let mut reader = BufReader::new(reader);
        loop {
            let mut line = Vec::new();
            match reader.read_until(options.line_terminator, &mut line) {
                Ok(count) if count > 0 => self.process_line(line)?,
                Ok(_) => return Ok(()),
                Err(err) => return Err(Error(format!("I/O error: {}", err), 1)),
            }
        }
    }
}

// A byte cutter that will cut out bytes by position in the line.
struct Bytes {
    range_set: RangeSet,
}

impl Bytes {
    fn new(range_set: RangeSet, _matches: &ArgMatches) -> Self {
        Bytes { range_set }
    }
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
    fn new(range_set: RangeSet, _matches: &ArgMatches) -> Self {
        Chars { range_set }
    }
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
    fn new(range_set: RangeSet, matches: &ArgMatches) -> Result<Self> {
        let idelim = matches.value_of("input-delimiter").unwrap_or("\t");
        if idelim.len() != 1 {
            return Err(Error("single character for delimiter".to_string(), 2));
        }

        let odelim = matches.value_of("output-delimiter").unwrap_or(idelim);
        if odelim.len() != 1 {
            return Err(Error("single character for delimiter".to_string(), 2));
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
        let cutter = Bytes::new(range_set, matches);
        Ok(Box::new(cutter))
    } else if let Some(rng) = matches.value_of("chars") {
        let mut range_set = RangeSet::from_string(rng)?;
        if options.complement {
            range_set.complement();
        }
        let cutter = Chars::new(range_set, matches);
        Ok(Box::new(cutter))
    } else if let Some(rng) = matches.value_of("fields") {
        let mut range_set = RangeSet::from_string(rng)?;
        if options.complement {
            range_set.complement();
        }
        let cutter = Fields::new(range_set, matches)?;
        Ok(Box::new(cutter))
    } else {
        Err(Error("not possible to select cutter".to_string(), 1))
    }
}
