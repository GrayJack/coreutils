use std::{
    io::{self, Write},
    iter::Peekable,
    process,
    str::Chars,
};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("echo.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let strings: Vec<String> = {
        // Safe to unwrap since we said it is required on clap yaml
        let values = matches.values_of("STRING").unwrap();
        let mut v = Vec::new();
        for value in values {
            v.push(value.to_owned());
        }
        v
    };

    match echo(&strings, matches.is_present("escape"), matches.is_present("no_newline")) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("echo: Failed to write to stdout.\n{}", e);
            process::exit(1);
        },
    };
}

/// Print given `strings` to standard output.
/// If `scape` true, it also prints the scape codes inside `strings`.
/// If `no_newline` true, it does not print a newline after.
fn echo(strings: &[String], escape: bool, no_newline: bool) -> io::Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();

    for (i, string) in strings.iter().enumerate() {
        if i > 0 {
            write!(output, " ")?;
        }
        if escape {
            let should_stop = print_escape(string, &mut output)?;
            if should_stop {
                break;
            }
        } else {
            write!(output, "{}", string)?;
        }
    }

    if !no_newline {
        writeln!(output)?;
    }

    Ok(())
}

/// Parse a `input` code from `base` code to a UTF-8 char.
/// The `max_digits` limits how many digits the `input` code can have.
fn parse_code(
    input: &mut Peekable<Chars>, base: u32, max_digits: u32, bits_per_digit: u32,
) -> Option<char> {
    use std::char::from_u32;

    let mut ret = 0x8000_0000;
    for _ in 0..max_digits {
        match input.peek().and_then(|c| c.to_digit(base)) {
            Some(n) => ret = (ret << bits_per_digit) | n,
            None => break,
        }
        input.next();
    }
    from_u32(ret)
}

/// Print the scape codes from `string`.
/// `output` is where it is going to be printed.
fn print_escape(string: &str, mut output: impl Write) -> io::Result<bool> {
    let mut stop = false;
    let mut buff = ['\\'; 2];
    let mut iter = string.chars().peekable();
    while let Some(mut c) = iter.next() {
        let mut start_at = 1;

        if c == '\\' {
            if let Some(n) = iter.next() {
                c = match n {
                    '\\' => '\\',
                    'a' => '\x07',
                    'b' => '\x08',
                    'c' => {
                        stop = true;
                        break;
                    },
                    'e' => '\x1b',
                    'f' => '\x0c',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'v' => '\x0b',
                    'x' => parse_code(&mut iter, 16, 2, 4).unwrap_or_else(|| {
                        start_at = 0;
                        n
                    }),
                    '0' => parse_code(&mut iter, 8, 3, 3).unwrap_or_else(|| {
                        start_at = 0;
                        n
                    }),
                    _ => {
                        start_at = 0;
                        n
                    },
                }
            }
        }

        buff[1] = c;

        for ch in &buff[start_at..] {
            write!(output, "{}", ch)?;
        }
    }

    Ok(stop)
}
