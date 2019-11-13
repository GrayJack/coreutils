use std::{
    fs::File,
    io::{self, prelude::*, BufReader},
};

use clap::{load_yaml, App, AppSettings::ColoredHelp, ArgMatches};

#[cfg(test)]
mod tests;

fn main() {
    let yaml = load_yaml!("wc.yml");
    let matches = App::from_yaml(yaml)
        .settings(&[ColoredHelp])
        .help_message("Display help information")
        .version_message("Display version information")
        .get_matches();

    let flags = WcFlags::from_matches(&matches);

    let filenames: Vec<String> = if let Some(values) = matches.values_of("FILE") {
        values.map(|v| v.to_string()).collect()
    } else {
        vec![String::from("-")]
    };

    let mut total_result = WcResult::default();
    for filename in &filenames {
        let result = if filename == "-" {
            let stdin = io::stdin();
            wc(stdin.lock())
        } else {
            match File::open(filename) {
                Ok(file) => wc(file),
                Err(err) => Err(err),
            }
        };

        match result {
            Err(err) => eprintln!("wc: {}: {}", filename, err),
            Ok(result) => {
                println!("{}", get_formatted_result(filename, &result, flags));
                total_result = total_result.combine(result);
            },
        }
    }

    if filenames.len() > 1 {
        println!("{}", get_formatted_result("total", &total_result, flags));
    }
}

#[derive(Default, Copy, Clone)]
struct WcFlags {
    print_bytes: bool,
    print_chars: bool,
    print_lines: bool,
    print_words: bool,
    print_max_line_len: bool,
    pretty: bool,
}

impl WcFlags {
    fn new() -> Self {
        WcFlags {
            print_bytes: false,
            print_chars: true,
            print_lines: true,
            print_words: true,
            print_max_line_len: false,
            pretty: false,
        }
    }

    fn from_matches(matches: &ArgMatches<'_>) -> Self {
        let print_bytes = matches.is_present("bytes");
        let print_chars = matches.is_present("chars");
        let print_lines = matches.is_present("lines");
        let print_words = matches.is_present("words");
        let print_max_line_len = matches.is_present("max-line-length");
        let pretty = matches.is_present("pretty");

        if !print_bytes
            && !print_chars
            && !print_lines
            && !print_words
            && !print_max_line_len
            && !pretty
        {
            return Self::new();
        }

        if pretty
            && !print_chars
            && !print_lines
            && !print_words
            && !print_max_line_len
            && !print_bytes
        {
            let mut flags = Self::new();
            flags.pretty = true;
            return flags;
        }

        WcFlags { print_bytes, print_chars, print_lines, print_words, print_max_line_len, pretty }
    }
}

#[derive(Default, Clone, Copy)]
struct WcResult {
    lines: u64,
    words: u64,
    chars: u64,
    bytes: u64,
    max_line_len: u32,
}

impl WcResult {
    fn combine(self, other: WcResult) -> Self {
        WcResult {
            lines: self.lines + other.lines,
            words: self.words + other.words,
            chars: self.chars + other.chars,
            bytes: self.bytes + other.bytes,
            max_line_len: self.max_line_len.max(other.max_line_len),
        }
    }
}

fn wc<R: Read>(stream: R) -> Result<WcResult, io::Error> {
    let reader = BufReader::new(stream);

    let mut result = WcResult::default();

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

fn get_formatted_result(filename: &str, result: &WcResult, flags: WcFlags) -> String {
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

    let push = if flags.pretty { push_pretty_res } else { push_unpretty_res };

    if flags.pretty {
        s.push_str(if filename == "-" { "(stdin)" } else { filename });
    }

    if flags.print_lines {
        push("lines", &mut s, result.lines);
    }
    if flags.print_words {
        push("words", &mut s, result.words);
    }
    if flags.print_chars {
        push("characters", &mut s, result.chars);
    }
    if flags.print_bytes {
        push("bytes", &mut s, result.bytes);
    }
    if flags.print_max_line_len {
        push("max line length", &mut s, result.max_line_len.into());
    }

    if filename != "-" && !flags.pretty {
        s.push_str(filename);
    }
    s
}
