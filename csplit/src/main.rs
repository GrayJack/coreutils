use std::{
    fmt,
    fs::{remove_file, File},
    io::{self, stdin, BufRead, BufReader, Read, Write},
    process, result,
};

use clap::ArgMatches;
use regex::Regex;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let mut created = Vec::new();

    match csplit(&matches, &mut created) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("csplit: {}", err);
            if !matches.is_present("keep") {
                for fname in created {
                    match remove_file(&fname) {
                        Ok(_) => (),
                        Err(err) => eprintln!("csplit: remove file {} failed: {}", fname, err),
                    }
                }
            }
            process::exit(1);
        },
    };
}

fn csplit(args: &ArgMatches, created: &mut Vec<String>) -> Result<()> {
    // Ok to unwrap because it has default value
    let prefix = args.value_of("prefix").unwrap();
    let silent = args.is_present("silent");
    // Ok to unwrap because it is required
    let filename = args.value_of("FILE").unwrap();
    // Ok to unwrap because it has default value
    let digits = parse_number(args.value_of("digits").unwrap())?;
    let mut reader = open_input(filename)?;
    let mut filer = Filer::new(prefix, digits, created, silent)?;

    if let Some(patterns) = args.values_of("PATTERN") {
        let mut patterns = build_patterns(patterns.collect::<Vec<_>>())?;
        patterns.push(Pattern::new(Box::new(NeverMatcher::new())));
        patterns.reverse();

        // We know that there is at least one pattern in the list so
        // we can use unwrap.
        let mut pattern = patterns.pop().unwrap();
        let mut lineno = 0;
        loop {
            let mut buffer = String::new();
            match reader.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => (),
                Err(err) => return Err(Error::ReadFailed(filename.to_string(), err)),
            }

            lineno += 1;

            let (rotate_file, rotate_pattern) = pattern.match_line(&mut filer, lineno, &buffer);

            if rotate_pattern {
                // We know that there is at least one more pattern
                // in the list so we can use unwrap.
                pattern = patterns.pop().unwrap();
            }

            if rotate_file {
                filer.rotate()?;
                let lines: Vec<_> = filer.buffer.drain(..).collect();
                for line in lines {
                    filer.write_line(&line)?;
                }
            }

            pattern.process_line(&mut filer, lineno, &buffer)?;
        }
        filer.flush()
    } else {
        Err(Error::MissingOperand(filename.to_string()))
    }
}

fn open_input(filename: &str) -> Result<BufReader<Box<dyn Read>>> {
    let input: Box<dyn Read> = if filename == "-" {
        Box::new(stdin())
    } else {
        Box::new(File::open(filename).map_err(|err| Error::OpenFailed(filename.to_string(), err))?)
    };

    Ok(BufReader::new(input))
}

fn build_patterns(patterns: Vec<&str>) -> Result<Vec<Pattern>> {
    let mut result = Vec::new();
    for pattern in patterns {
        match pattern.chars().clone().next() {
            Some('0'..='9') => result.push(Pattern::new(LineMatcher::parse(&pattern)?)),
            Some(ch @ '/') | Some(ch @ '%') => {
                result.push(Pattern::new(RegexMatcher::parse(&pattern, ch)?))
            },

            Some('{') => {
                if let Some(pat) = result.last_mut() {
                    pat.repeat = parse_repeat(&pattern)?;
                } else {
                    return Err(Error::InvalidPattern(pattern.to_string()));
                }
            },

            _ => {
                return Err(Error::InvalidPattern(pattern.to_string()));
            },
        }
    }
    Ok(result)
}

/// Output files handler.
///
/// The filer will rotate the files on request, write lines to the
/// current file, and keep track of created output files.
struct Filer<'a> {
    silent: bool,
    prefix: &'a str,
    digits: i32,
    file_counter: i32,
    created: &'a mut Vec<String>,
    writer: File,
    bytes: usize,
    current: String,
    pub buffer: Vec<String>,
}

fn create_file(current: &str, created: &mut Vec<String>) -> Result<File> {
    match File::create(current) {
        Ok(file) => {
            created.push(current.to_string());
            Ok(file)
        },
        Err(err) => Err(Error::CreateFailed(current.to_string(), err)),
    }
}

impl<'a> Filer<'a> {
    fn new(
        prefix: &'a str, digits: i32, created: &'a mut Vec<String>, silent: bool,
    ) -> Result<Filer<'a>> {
        let current = format!("{0}{2:01$}", prefix, digits as usize, 0);
        let writer = create_file(&current, created)?;
        Ok(Filer {
            current,
            writer,
            prefix,
            digits,
            created,
            silent,
            bytes: 0,
            file_counter: 1,
            buffer: Vec::new(),
        })
    }

    fn rotate(&mut self) -> Result<()> {
        self.current = format!("{0}{2:01$}", self.prefix, self.digits as usize, self.file_counter);
        if !self.silent {
            println!("{}", self.bytes);
        }
        self.bytes = 0;
        self.file_counter += 1;
        create_file(&self.current, self.created).map(|file| self.writer = file)
    }

    fn write_line(&mut self, line: &str) -> Result<()> {
        self.bytes += line.len();
        self.writer
            .write_all(line.as_bytes())
            .map_err(|err| Error::WriteFailed(self.current.clone(), err))
    }

    fn flush(&mut self) -> Result<()> {
        if !self.silent {
            println!("{}", self.bytes);
        }
        Ok(())
    }
}

#[derive(Debug)]
enum Error {
    OutOfRange(usize, Option<i32>),
    CreateFailed(String, io::Error),
    OpenFailed(String, io::Error),
    ReadFailed(String, io::Error),
    WriteFailed(String, io::Error),
    InvalidNumber(String),
    InvalidPattern(String),
    MissingOperand(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::OutOfRange(line, None) => write!(f, "'{}': line number out of range", line),
            Error::OutOfRange(line, Some(rep)) => {
                write!(f, "'{}': line number out of range on repetition {}", line, rep)
            },
            Error::CreateFailed(file, err) => {
                write!(f, "cannot open '{}' for writing: {}", file, err)
            },
            Error::OpenFailed(file, err) => {
                write!(f, "cannot open '{}' for reading: {}", file, err)
            },
            Error::ReadFailed(file, err) => write!(f, "cannot read '{}': {}", file, err),
            Error::WriteFailed(file, err) => write!(f, "cannot write '{}': {}", file, err),
            Error::InvalidNumber(val) => write!(f, "invalid number '{}'", val),
            Error::InvalidPattern(pat) => write!(f, "{}: invalid pattern", pat),
            Error::MissingOperand(arg) => write!(f, "missing operand after '{}'", arg),
        }
    }
}

type Result<T> = result::Result<T, Error>;

/// Pattern with a matcher and a repeat.
struct Pattern {
    /// Number of repetitions. Negative or zero repeat means infinite
    /// repetition
    repeat: i32,

    /// Count of number of matches.
    count: i32,

    matcher: Box<dyn Matcher>,
}

impl Pattern {
    fn new(matcher: Box<dyn Matcher>) -> Pattern { Pattern { repeat: 1, count: 1, matcher } }

    /// Check if line matches.
    ///
    /// Returns a pair of booleans. First boolean indicate the the
    /// file should be rotated, second one if the pattern should be
    /// rotated.
    fn match_line(&mut self, _filer: &mut Filer, lineno: usize, line: &str) -> (bool, bool) {
        let (rotate_file, pattern_matched) = self.matcher.match_line(lineno, line);

        if pattern_matched {
            self.repeat -= 1;
            self.count += 1;
            (rotate_file, self.repeat == 0)
        } else {
            (false, false)
        }
    }

    /// Process a line of input.
    ///
    /// Return `true` if the pattern is exhausted, `false` otherwise.
    fn process_line(&mut self, filer: &mut Filer, lineno: usize, line: &str) -> Result<()> {
        match self.matcher.process_line(filer, lineno, line) {
            Err(Error::OutOfRange(l, None)) => Err(Error::OutOfRange(l, Some(self.count))),
            err => err,
        }
    }
}

/// Matcher trait used to match lines.
trait Matcher: fmt::Debug {
    /// Check if a line matches.
    ///
    /// Return a pair of booleans:
    /// - First boolean is `true` if the file rotation should take place.
    /// - Second boolean is `true` if the line matches.
    fn match_line(&mut self, lineno: usize, line: &str) -> (bool, bool);

    /// Process an input line.
    ///
    /// Return `true` if the matcher matched the line, `false`
    /// otherwise.
    fn process_line(&mut self, filer: &mut Filer, lineno: usize, line: &str) -> Result<()>;
}

/// Never match a line.
///
/// Used as last pattern when reading to write the rest of the file to
/// a separate file. It will just write lines to the output file,
/// never rotate, and never exhaust.
#[derive(Debug)]
struct NeverMatcher;

impl NeverMatcher {
    fn new() -> NeverMatcher { NeverMatcher {} }
}

impl Matcher for NeverMatcher {
    fn match_line(&mut self, _lineno: usize, _line: &str) -> (bool, bool) { (false, false) }

    fn process_line(&mut self, filer: &mut Filer, _lineno: usize, line: &str) -> Result<()> {
        filer.write_line(line)
    }
}

/// Line matcher.
///
/// Match a specific line count relative to the start of the section.
#[derive(Debug)]
struct LineMatcher {
    lineno: usize,
}

impl LineMatcher {
    fn parse(pattern: &str) -> Result<Box<dyn Matcher>> {
        let num =
            pattern.parse::<usize>().map_err(|_| Error::InvalidPattern(pattern.to_string()))?;
        Ok(Box::new(LineMatcher { lineno: num }))
    }
}

impl Matcher for LineMatcher {
    fn match_line(&mut self, lineno: usize, _line: &str) -> (bool, bool) {
        if self.lineno == lineno { (true, true) } else { (false, false) }
    }

    fn process_line(&mut self, filer: &mut Filer, lineno: usize, line: &str) -> Result<()> {
        if self.lineno < lineno {
            Err(Error::OutOfRange(self.lineno, None))
        } else {
            filer.write_line(line)
        }
    }
}

/// Regexp matcher.
///
/// Match a line if it matches the regular expression.
#[derive(Debug)]
struct RegexMatcher {
    regex:      Regex,
    skip:       bool,
    offset:     i32,
    line_match: Option<usize>,
}

// /REGEXP/[OFFSET]
// %REGEXP%[OFFSET]
impl RegexMatcher {
    fn new(regex: &str, skip: bool, offset: i32) -> Result<RegexMatcher> {
        let regex = Regex::new(regex).map_err(|_| Error::InvalidPattern(regex.to_string()))?;
        Ok(RegexMatcher { skip, regex, offset, line_match: None })
    }

    fn parse(pattern: &str, first: char) -> Result<Box<dyn Matcher>> {
        let mut chars = pattern.chars().enumerate();
        let pat_end = match chars.by_ref().skip(1).find(move |&(_, c)| c == first) {
            Some((pos, _)) => pos,
            None => {
                return Err(Error::InvalidPattern(pattern.to_string()));
            },
        };

        let slice = &pattern[pat_end + 1..];
        let offset = if !slice.is_empty() { parse_number(slice)? } else { 0 };

        Ok(Box::new(RegexMatcher::new(&pattern[1..pat_end], first == '%', offset)?))
    }
}

impl Matcher for RegexMatcher {
    fn match_line(&mut self, lineno: usize, line: &str) -> (bool, bool) {
        if let Some(the_line) = self.line_match {
            return (!self.skip, the_line == lineno);
        }

        if self.regex.is_match(line) {
            if self.offset > 0 {
                self.line_match = Some(lineno + self.offset as usize);
                (!self.skip, false)
            } else {
                (!self.skip, true)
            }
        } else {
            (!self.skip, false)
        }
    }

    fn process_line(&mut self, filer: &mut Filer, _lineno: usize, line: &str) -> Result<()> {
        if self.offset < 0 {
            filer.buffer.push(line.to_string());
            let count = filer.buffer.len() as i32 + self.offset;
            if count > 0 {
                let lines: Vec<_> = filer.buffer.drain(0..count as usize).collect();
                for line in lines {
                    filer.write_line(&line)?;
                }
            }
        } else if !self.skip {
            filer.write_line(line)?;
        }
        Ok(())
    }
}

// {INTEGER}
// {*}
fn parse_repeat(pattern: &str) -> Result<i32> {
    if pattern == "{*}" {
        Ok(-1)
    } else {
        let len = pattern.len() - 1;
        match &pattern[1..len].parse() {
            Ok(value) => Ok(*value),
            Err(_) => Err(Error::InvalidPattern(pattern.to_string())),
        }
    }
}

/// Parse a string slice as a number, or return error.
fn parse_number(slice: &str) -> Result<i32> {
    if slice.is_empty() {
        slice.parse().map_err(|_| Error::InvalidNumber(slice.to_string()))
    } else {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        ($xpr:expr, $pat:pat if $cond:expr) => {
            match $xpr {
                $pat if $cond => true,
                ref xpr => panic!(
                    "assert_matches: '{:?} if {}' doesn't match '{}'",
                    xpr,
                    stringify!($cond),
                    stringify!($pat)
                ),
            }
        };
    }

    #[test]
    fn regex_matcher() {
        assert_matches!(RegexMatcher::parse("/foo/", '/'), Ok(_));
        assert_matches!(RegexMatcher::parse("/foo%", '/'), Err(_));
        assert_matches!(RegexMatcher::parse("%foo%", '%'), Ok(_));
        assert_matches!(RegexMatcher::parse("%foo/", '%'), Err(_));
        assert_matches!(RegexMatcher::parse("/foo/0", '/'), Ok(_));
        assert_matches!(RegexMatcher::parse("/foo/1", '/'), Ok(_));
        assert_matches!(RegexMatcher::parse("/foo/-1", '/'), Ok(_));
    }
}
