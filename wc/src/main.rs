use std::{
    fs::File,
    io::{self, prelude::*, BufReader},
};

use clap::{load_yaml, App, ArgMatches};

const F_PRINT_LINES: u8 = 0x1;
const F_PRINT_WORDS: u8 = 0x2;
const F_PRINT_CHARS: u8 = 0x4;
const F_PRINT_BYTES: u8 = 0x8;
const F_PRINT_MAX_LINE_LEN: u8 = 0x10;
const F_PRETTY_PRINT: u8 = 0x20;
const DEFAULT_FLAGS: u8 = F_PRINT_LINES | F_PRINT_WORDS | F_PRINT_CHARS;

fn main() {
    let yaml = load_yaml!("wc.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let filenames: Vec<String> = {
        if let Some(values) = matches.values_of("FILE") {
            let mut v = Vec::new();
            for value in values {
                v.push(value.to_owned());
            }
            v
        } else {
            vec![String::from("-")]
        }
    };

    let flags = parse_flags(&matches);

    let mut total_result = WcResult::default();
    for filename in &filenames {
        let result = if filename == "-" {
            let stdin = io::stdin();
            wc(stdin.lock(), flags)
        } else {
            match File::open(filename) {
                Ok(file) => wc(file, flags),
                Err(err) => Err(err),
            }
        };

        match result {
            Err(err) => eprintln!("wc: {}: {}", filename, err),
            Ok(result) => {
                println!("{}", get_formatted_result(filename, &result));
                total_result = total_result.combine(result);
            },
        }
    }

    if filenames.len() > 1 {
        println!("{}", get_formatted_result("total", &total_result));
    }
}

#[derive(Default)]
struct WcResult {
    pub lines: u64,
    pub words: u64,
    pub chars: u64,
    pub bytes: u64,
    pub max_line_len: u32,
    pub flags: u8,
}

impl WcResult {
    pub fn combine(self, other: WcResult) -> WcResult {
        WcResult {
            lines: self.lines + other.lines,
            words: self.words + other.words,
            chars: self.chars + other.chars,
            bytes: self.bytes + other.bytes,
            max_line_len: self.max_line_len.max(other.max_line_len),
            flags: self.flags | other.flags,
        }
    }
}

fn wc<R: Read>(stream: R, flags: u8) -> Result<WcResult, io::Error> {
    let reader = BufReader::new(stream);

    let mut result = WcResult { flags, ..Default::default() };

    for line in reader.lines() {
        let line = line?;

        result.lines += 1;

        let mut last_was_whitespace = true;
        let mut n_chars_excluding_newline = 0;
        for chr in line.chars() {
            n_chars_excluding_newline += 1;

            let is_whitespace = chr.is_whitespace();
            if is_whitespace && !last_was_whitespace {
                result.words += 1;
            }
            last_was_whitespace = is_whitespace;
        }

        // The max line length considers characters, not bytes.
        result.max_line_len = result.max_line_len.max(n_chars_excluding_newline);

        result.chars += u64::from(n_chars_excluding_newline) + 1;

        if !last_was_whitespace {
            result.words += 1;
        }

        result.bytes += line.len() as u64 + 1;
    }

    Ok(result)
}

fn get_formatted_result(filename: &str, result: &WcResult) -> String {
    let flags = result.flags;
    let mut s = String::with_capacity(64);

    fn push_unpretty_res<T: ToString>(_name: &str, out: &mut String, result: T) {
        out.push_str(&result.to_string());
        out.push(' ');
    }

    fn push_pretty_res<T: ToString>(name: &str, out: &mut String, result: T) {
        out.push_str("\n  ");
        out.push_str(name);
        out.push_str(": ");
        out.push_str(&result.to_string());
    }

    let pretty_print = (flags & F_PRETTY_PRINT) != 0;

    let push = if pretty_print { push_pretty_res } else { push_unpretty_res };

    if pretty_print {
        s.push_str(if filename == "-" { "(stdin)" } else { filename });
    }

    if (flags & F_PRINT_LINES) != 0 {
        push("lines", &mut s, result.lines);
    }
    if (flags & F_PRINT_WORDS) != 0 {
        push("words", &mut s, result.words);
    }
    if (flags & F_PRINT_CHARS) != 0 {
        push("characters", &mut s, result.chars);
    }
    if (flags & F_PRINT_BYTES) != 0 {
        push("bytes", &mut s, result.bytes);
    }
    if (flags & F_PRINT_MAX_LINE_LEN) != 0 {
        push("max line length", &mut s, result.max_line_len.into());
    }

    if filename != "-" && !pretty_print {
        s.push_str(filename);
    }
    s
}

fn parse_flags(matches: &ArgMatches<'_>) -> u8 {
    let mut flags = 0;

    if matches.is_present("bytes") {
        flags |= F_PRINT_BYTES;
    }
    if matches.is_present("chars") {
        flags |= F_PRINT_CHARS;
    }
    if matches.is_present("lines") {
        flags |= F_PRINT_LINES;
    }
    if matches.is_present("max-line-length") {
        flags |= F_PRINT_MAX_LINE_LEN;
    }
    if matches.is_present("words") {
        flags |= F_PRINT_WORDS;
    }

    flags = if flags == 0 { DEFAULT_FLAGS } else { flags };

    if matches.is_present("pretty") {
        flags |= F_PRETTY_PRINT;
    }

    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestReader<'a> {
        buf: &'a str,
        i:   usize,
    }

    impl<'a> TestReader<'a> {
        pub fn new(s: &'a str) -> Self { TestReader { buf: s, i: 0 } }
    }

    impl Read for TestReader<'_> {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            let i = self.i;
            let n = out.len().min(self.buf.len() - i);
            let buf_ptr = self.buf.as_ptr();
            let out_ptr = out.as_mut_ptr();
            unsafe {
                buf_ptr.copy_to(out_ptr, n);
            }
            self.i += n;
            Ok(n)
        }
    }

    #[test]
    fn wc_stdin() {
        let test_str = TestReader::new("This is a test string");
        let res = get_formatted_result(
            "-",
            &wc(
                test_str,
                F_PRINT_BYTES
                    | F_PRINT_CHARS
                    | F_PRINT_LINES
                    | F_PRINT_WORDS
                    | F_PRINT_MAX_LINE_LEN,
            )
            .unwrap(),
        );
        assert_eq!(res, String::from("1 5 22 22 21 "));
    }

    #[test]
    fn wc_pretty_print() {
        let test_str = TestReader::new("This is a test string");
        let res = get_formatted_result(
            "test",
            &wc(test_str, F_PRINT_BYTES | F_PRINT_LINES | F_PRETTY_PRINT).unwrap(),
        );
        assert_eq!(
            res,
            String::from(
                "test
  lines: 1
  bytes: 22"
            )
        );
    }
}
