use std::{
    fs::File,
    io::{self, BufRead, BufReader, ErrorKind, Read, Write},
    process,
};

mod cli;


fn main() {
    let matches = cli::create_app().get_matches();
    let input_filename = matches.value_of("INPUT").unwrap_or("-");
    let output_filename = matches.value_of("OUTPUT").unwrap_or("-");
    let flags = Flags::from_matches(&matches);

    let reader: Box<dyn io::Read> = if input_filename == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(input_filename).unwrap_or_else(|err| {
            eprintln!("uniq: Cannot open '{}' for reading: {}.", input_filename, err);
            process::exit(1);
        }))
    };
    let mut reader = BufReader::new(reader);

    let mut writer: Box<dyn io::Write> = if output_filename == "-" {
        Box::new(io::stdout())
    } else {
        Box::new(File::create(output_filename).unwrap_or_else(|err| {
            eprintln!("uniq: Cannot create '{}' for writing: {}.", output_filename, err);
            process::exit(1);
        }))
    };

    if let Err(_) = uniq(&mut reader, &mut writer, flags) {
        process::exit(1);
    };
}


#[derive(Default)]
struct Flags {
    show_count:       bool,        // -c | --show_count
    supress_unique:   bool,        // -d | --repeated
    supress_repeated: bool,        // -u | --unique
    skip_chars:       Option<u64>, // -s | --skip-chars=N
    skip_fields:      Option<u64>, // -f | --skip-fields=N
}

impl Flags {
    fn from_matches(matches: &clap::ArgMatches) -> Self {
        // Used to capture skip_chars and skip_fields
        let parse_arg_to_u64 = |arg: Option<&str>, error_msg: &str| -> Option<u64> {
            if let Some(arg) = arg {
                let number = arg.parse::<u64>().unwrap_or_else(|_| {
                    eprintln!("uniq: {}.", error_msg);
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
            skip_chars:       parse_arg_to_u64(
                matches.value_of("skip-chars"),
                "Invalid number of bytes to skip",
            ),
            skip_fields:      parse_arg_to_u64(
                matches.value_of("skip-fields"),
                "Invalid number of fields to skip",
            ),
        }
    }
}


fn uniq<R: Read, W: Write>(
    reader: &mut BufReader<R>, writer: &mut W, flags: Flags,
) -> Result<(), io::Error> {
    // Always compared against current_line
    let mut last_line = String::new();
    let mut last_line_count: u64 = 0;

    // If --skip-chars
    let bytes_to_skip = flags.skip_chars.unwrap_or(0) as usize;
    let mut bytes_to_skip: Vec<u8> = vec![0u8; bytes_to_skip];

    // Loop for each line read
    loop {
        let mut current_line = String::new();

        if flags.skip_chars.is_some() {
            let skip_result = reader.read_exact(&mut bytes_to_skip[..]);
            // Check error or EOF
            match skip_result {
                Ok(_) => {},
                Err(err) => {
                    if let ErrorKind::UnexpectedEof = err.kind() {
                        // Ignore
                    } else {
                        return Err(err);
                    }
                },
            }
        }

        let size = reader.read_line(&mut current_line);

        let mut should_exit = false;
        // Check error or EOF
        match size {
            Err(err) => {
                eprintln!("uniq: Input error: {}.", err);
                return Err(err);
            },
            // EOF, exit after this loop
            Ok(0) => should_exit = true,
            // Keep looping
            Ok(_) => {},
        }

        let line_changed = current_line != last_line;
        let current_line_count = if line_changed { 1 } else { last_line_count + 1 };

        // The combination of these two flags supress all output
        if flags.supress_repeated && flags.supress_unique {
            if should_exit {
                break;
            }
            continue;
        }

        // The following block decides if current_line or last_line should be shown
        // The lines are always shown as early as possible
        // Output formatting is different depending on flags.show_count
        if flags.show_count {
            if line_changed {
                let mut should_show_last_line = false;

                if flags.supress_unique {
                    if last_line_count > 1 {
                        should_show_last_line = true;
                    }
                } else if flags.supress_repeated {
                    if last_line_count == 1 {
                        should_show_last_line = true;
                    }
                } else {
                    should_show_last_line = true;
                }

                if should_show_last_line && last_line_count > 0 {
                    write!(writer, "{:7} ", last_line_count)?;
                    writer.write_all(last_line.as_bytes())?;
                }
            }
        } else {
            let mut line_to_show: Option<&str> = None;

            if flags.supress_unique {
                if current_line_count == 2 {
                    line_to_show = Some(&current_line);
                }
            } else if flags.supress_repeated {
                if line_changed && last_line_count == 1 {
                    line_to_show = Some(&last_line);
                }
            } else {
                if line_changed {
                    line_to_show = Some(&current_line);
                }
            }

            if let Some(line) = line_to_show {
                writer.write_all(line.as_bytes())?;
            }
        }

        last_line = current_line;
        last_line_count = current_line_count;
        if should_exit {
            break;
        }
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
        let expected = "A\nB\nC\nD";
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
        assert_eq!("ABC", test_uniq("ABC", flags_none()));
    }

    #[test]
    fn test_uniq_empty() {
        assert_eq!("", test_uniq("", flags_none()));
    }

    #[test]
    fn test_uniq_flag_count() {
        let input = "A\nA\nB\nC\nC\nD";
        let expected = "      2 A\n      1 B\n      2 C\n      1 D";
        let flags = Flags { show_count: true, ..flags_none() };
        assert_eq!(expected, test_uniq(input, flags));
    }

    #[test]
    fn test_uniq_flag_unique() {
        let input = "A\nA\nB\nC\nC\nD";
        let expected = "B\nD";
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
        let input = "_A\n_A\n_B\n_C\n_C\n_D";
        let expected = "A\nB\nC\nD";
        let flags = Flags { skip_chars: Some(1), ..flags_none() };
        assert_eq!(expected, test_uniq(input, flags));
    }

    #[test]
    fn test_uniq_combined_flags_count_and_unique() {
        let expected = "      1 B\n      1 D";
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
