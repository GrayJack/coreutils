use std::{
    cmp,
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    process,
};

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();
    let input_filename = matches.value_of("INPUT").unwrap_or("-");
    let output_filename = matches.value_of("OUTPUT").unwrap_or("-");
    let flags = Flags::from_matches(&matches);

    let mut reader: Box<dyn BufRead> = if input_filename == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        Box::new(BufReader::new(File::open(input_filename).unwrap_or_else(|err| {
            eprintln!("uniq: Cannot open '{}' for reading: {}.", input_filename, err);
            process::exit(1);
        })))
    };

    let mut writer: Box<dyn Write> = if output_filename == "-" {
        Box::new(BufWriter::new(io::stdout()))
    } else {
        Box::new(BufWriter::new(File::create(output_filename).unwrap_or_else(|err| {
            eprintln!("uniq: Cannot create '{}' for writing: {}.", output_filename, err);
            process::exit(1);
        })))
    };

    uniq(&mut reader, &mut writer, flags).unwrap_or_else(|err| {
        eprintln!("uniq: {}.", err);
        process::exit(1);
    });
}

// -f is ignored before -s
#[derive(Default)]
struct Flags {
    show_count:       bool,          // -c | --show_count
    supress_unique:   bool,          // -d | --repeated
    supress_repeated: bool,          // -u | --unique
    skip_bytes:       Option<usize>, // -s | --skip-chars=N
    skip_fields:      Option<usize>, // -f | --skip-fields=N
}
// skip_utf8_chars:  Option<usize>, // --skip-utf8=N

impl Flags {
    fn from_matches(matches: &clap::ArgMatches) -> Self {
        // Used to capture skip_bytes and skip_fields
        let try_parse_arg_to_usize = |arg: Option<&str>, error_msg| {
            if let Some(arg) = arg {
                let number = arg.parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("uniq: {} '{}'.", error_msg, arg);
                    process::exit(1);
                });
                Some(number)
            } else {
                None
            }
        };

        Flags {
            show_count:       matches.is_present("show_count"),
            supress_unique:   matches.is_present("repeated"),
            supress_repeated: matches.is_present("unique"),
            skip_bytes:       try_parse_arg_to_usize(
                matches.value_of("skip-chars"),
                "--skip-chars: Invalid number of bytes to skip",
            ),
            skip_fields:      try_parse_arg_to_usize(
                matches.value_of("skip-fields"),
                "--skip-fields: Invalid number of fields to skip",
            ),
        }
    }
}

// Return the total of bytes skipped
fn skip_fields_and_bytes(string: &str, fields: usize, bytes: usize) -> usize {
    let mut iter = string.char_indices().peekable();
    let mut skipped = 0;

    // Skip fields, regex is "\s*\S*"
    for _ in 0..fields {
        while let Some((char_bytes, character)) = iter.peek() {
            if character.is_whitespace() {
                break;
            }
            skipped += char_bytes;
            iter.next();
        }

        while let Some((char_bytes, character)) = iter.peek() {
            if !character.is_whitespace() {
                break;
            }
            skipped += char_bytes;
            iter.next();
        }
    }

    // Skip bytes
    // Try to skip them, but don't allow to overflow string.as_bytes().len()
    let skipped = cmp::min(string.as_bytes().len(), skipped + bytes);

    skipped
}

fn uniq<R: BufRead, W: Write>(
    reader: &mut R, writer: &mut W, flags: Flags,
) -> Result<(), io::Error> {
    // If -s and -f are unset, last_line is guaranteed to be equals to the previous line,
    // else, last_line is the first line in the last set of lines that with each other,
    // considering the ignored bytes or fields.
    let mut last_line = String::new();
    let mut last_skipped_bytes = 0;

    // Number of times that `last_line` matched
    let mut last_count = 0;

    // After reaching EOF, don't exit immediately, still process the `last_line`
    let mut reached_eof = false;

    // Loop for each line read
    while !reached_eof {
        let mut new_line = String::new();

        // Using `reader.read_line()` to capture the line break characters
        let bytes_read = reader.read_line(&mut new_line)?;
        reached_eof = bytes_read == 0;

        // Only happens on the last line of the input, when reaching EOF
        // Add \n at the end if necessary
        if bytes_read > 0 && new_line.bytes().next_back() != Some(b'\n') {
            new_line.reserve_exact(1);
            new_line.push('\n');
        }

        let skipped_bytes = skip_fields_and_bytes(
            new_line.as_str(),
            flags.skip_fields.unwrap_or(0),
            flags.skip_bytes.unwrap_or(0),
        );

        let new_slice = &new_line.as_bytes()[skipped_bytes..];
        let last_slice = &last_line.as_bytes()[last_skipped_bytes..];

        let line_changed = new_slice != last_slice;
        let current_line_match_count = if line_changed { 1 } else { last_count + 1 };

        // The combination of these two flags supress all output
        if flags.supress_repeated && flags.supress_unique {
            continue;
        }

        // The following block decides if new_line or last_line should be shown
        // The lines are always shown as early as possible
        // Formatting changes based on flags.show_count
        if flags.show_count {
            if line_changed {
                let mut should_show_last_line = false;

                if flags.supress_unique {
                    if last_count > 1 {
                        should_show_last_line = true;
                    }
                } else if flags.supress_repeated {
                    if last_count == 1 {
                        should_show_last_line = true;
                    }
                } else {
                    should_show_last_line = true;
                }

                if should_show_last_line && last_count > 0 {
                    write!(writer, "{:7} ", last_count)?;
                    writer.write_all(last_line.as_bytes())?;
                }
            }
        } else {
            let mut line_to_show: Option<&str> = None;

            if flags.supress_unique {
                if current_line_match_count == 2 {
                    line_to_show = Some(&new_line);
                }
            } else if flags.supress_repeated {
                if line_changed && last_count == 1 {
                    line_to_show = Some(&last_line);
                }
            } else if line_changed {
                line_to_show = Some(&new_line);
            }

            if let Some(line) = line_to_show {
                writer.write_all(line.as_bytes())?;
            }
        }

        last_line = new_line;
        last_count = current_line_match_count;
        last_skipped_bytes = skipped_bytes;
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    // Test utility, takes input and retrieves uniq's output
    fn test_uniq(input: &str, flags: Flags) -> String {
        let mut reader = BufReader::new(input.as_bytes());

        let mut output = Vec::<u8>::new();
        uniq(&mut reader, &mut output, flags).unwrap();

        String::from_utf8(output).unwrap()
    }

    // Test utility, return empty Flags (all set to false or None)
    fn flags_none() -> Flags { Flags::default() }

    #[test]
    fn test_uniq_basic_usage() {
        let input = "A\nA\nB\nC\nC\nD";
        let expected = "A\nB\nC\nD\n";
        assert_eq!(expected, test_uniq(input, flags_none()));
    }

    #[test]
    fn test_uniq_blank_lines() {
        let input = "\n\n\n \n \n";
        let expected = "\n \n";
        assert_eq!(expected, test_uniq(input, flags_none()));
    }

    #[test]
    fn test_uniq_without_line_break() {
        assert_eq!("ABC\n", test_uniq("ABC", flags_none()));
    }

    #[test]
    fn test_uniq_empty() {
        assert_eq!("", test_uniq("", flags_none()));
    }

    // #[test]
    // fn test_uniq_line_endings() {
    //     let input = "A\r\nA\n\n\r\n\n";
    //     let expected = "A\n\n";
    //     assert_eq!(expected, test_uniq(input, flags_none()));
    // }

    #[test]
    fn test_uniq_flag_count() {
        let input = "A\nA\nB\nC\nC\nD";
        let expected = "      2 A\n      1 B\n      2 C\n      1 D\n";
        let flags = Flags { show_count: true, ..flags_none() };
        assert_eq!(expected, test_uniq(input, flags));
    }

    #[test]
    fn test_uniq_flag_unique() {
        let input = "A\nA\nB\nC\nC\nD";
        let expected = "B\nD\n";
        let flags = Flags { supress_repeated: true, ..flags_none() };
        assert_eq!(expected, test_uniq(input, flags));
    }

    #[test]
    fn test_uniq_flag_repeated() {
        let input = "A\nA\nB\nC\nC\nD";
        let expected = "A\nC\n";
        let flags = Flags { supress_unique: true, ..flags_none() };
        assert_eq!(expected, test_uniq(input, flags));
    }

    #[test]
    fn test_uniq_flag_skip_chars() {
        let input = "qwe\neee\neeee\n\n0x11\n0b11";
        let expected = "qwe\neeee\n\n0x11\n";
        let flags = Flags { skip_bytes: Some(2), ..flags_none() };
        assert_eq!(expected, test_uniq(input, flags));
    }

    #[test]
    fn test_uniq_flag_skip_fields() {
        let input = "a a\na b\nb b\nc b";
        let expected = "a a\na b\n";
        let flags = Flags { skip_fields: Some(1), ..flags_none() };
        assert_eq!(expected, test_uniq(input, flags));
    }

    #[test]
    fn test_uniq_combined_flags_count_and_unique() {
        let expected = "      1 B\n      1 D\n";
        let input = "A\nA\nB\nC\nC\nD";
        let flags = Flags { show_count: true, supress_repeated: true, ..flags_none() };
        assert_eq!(expected, test_uniq(input, flags));
    }

    #[test]
    fn test_uniq_combined_flags_count_and_repeated() {
        let input = "A\nA\nB\nC\nC\nD";
        let expected = "      2 A\n      2 C\n";
        let flags = Flags { show_count: true, supress_unique: true, ..flags_none() };
        assert_eq!(expected, test_uniq(input, flags));
    }

    #[test]
    fn test_uniq_combined_flags_repeated_and_unique() {
        let input = " A \n A \n B \n C \n C \n D ";
        let expected = "";
        let flags = Flags { supress_unique: true, supress_repeated: true, ..flags_none() };
        assert_eq!(expected, test_uniq(input, flags));
    }
}
