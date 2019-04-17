use std::io::{self, Write};

use clap::{App, load_yaml};

fn main() -> Result<(), Box<std::error::Error>> {
    let yaml = load_yaml!("echo.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let strings: Vec<String> = {
        // Safe to unwrap since we said it is required on clap yaml
        let values = matches.values_of("STRING").unwrap();
        let mut v = Vec::new();
        for value in values {
            v.push(value.to_owned());
        }
        v
    };

    echo(strings, matches.is_present("escape"), matches.is_present("no_newline"))?;

    Ok(())
}

fn echo(strings: Vec<String>, escape: bool, no_newline: bool) -> io::Result<()> {
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
        }
        write!(output, "{}", string)?;
    }

    if !no_newline {
        writeln!(output)?;
    }

    Ok(())
}

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
                        break
                    },
                    'e' => '\x1b',
                    'f' => '\x0c',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'v' => '\x0b',
                    _ => {
                        start_at = 0;
                        n
                    }
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
