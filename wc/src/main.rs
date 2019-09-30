use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::process;

use clap::{load_yaml, App, ArgMatches};

const PRINT_LINES: u8 = 0x1;
const PRINT_WORDS: u8 = 0x2;
const PRINT_CHARS: u8 = 0x4;
const PRINT_BYTES: u8 = 0x8;
const PRINT_MAX_LINE_LEN: u8 = 0x10;
const DEFAULT_FLAGS: u8 = PRINT_LINES | PRINT_WORDS | PRINT_CHARS;

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
            Ok(result) => print_result(filename, &result),
        }
    }

    process::exit(0);
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

fn wc<R: Read>(stream: R, flags: u8) -> Result<WcResult, io::Error> {
    let reader = BufReader::new(stream);

    let mut result = WcResult {
        flags,
        ..Default::default()
    };

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

fn print_result(filename: &str, result: &WcResult) {
    let flags = result.flags;
    let mut s = String::with_capacity(64);

    // Order is: newline, word, character, byte, maximum line length.
    if (flags & PRINT_LINES) != 0 {
        s.push_str(&result.lines.to_string());
        s.push(' ');
    }
    if (flags & PRINT_WORDS) != 0 {
        s.push_str(&result.words.to_string());
        s.push(' ');
    }
    if (flags & PRINT_CHARS) != 0 {
        s.push_str(&result.chars.to_string());
        s.push(' ');
    }
    if (flags & PRINT_BYTES) != 0 {
        s.push_str(&result.bytes.to_string());
        s.push(' ');
    }
    if (flags & PRINT_MAX_LINE_LEN) != 0 {
        s.push_str(&result.max_line_len.to_string());
        s.push(' ');
    }

    s.push_str(filename);

    println!("{}", s);
}

fn parse_flags(matches: &ArgMatches<'_>) -> u8 {
    let mut flags = 0;

    if matches.is_present("bytes") {
        flags |= PRINT_BYTES;
    }
    if matches.is_present("chars") {
        flags |= PRINT_CHARS;
    }
    if matches.is_present("lines") {
        flags |= PRINT_LINES;
    }
    if matches.is_present("max-line-length") {
        flags |= PRINT_MAX_LINE_LEN;
    }
    if matches.is_present("words") {
        flags |= PRINT_WORDS;
    }

    if flags == 0 {
        DEFAULT_FLAGS
    } else {
        flags
    }
}
