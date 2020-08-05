use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read, Write},
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


struct Flags {
    show_count:       bool,        // -c | --show_count
    supress_unique:   bool,        // -d | --repeated
    supress_repeated: bool,        // -u | --unique
    skip_chars:       Option<u64>, // -s | --skip-chars=N
    skip_fields:      Option<u64>, // -f | --skip-fields=N
}

impl Flags {
    fn from_matches(matches: &clap::ArgMatches) -> Self {
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
                "invalid number of bytes to skip",
            ),
            skip_fields:      parse_arg_to_u64(
                matches.value_of("skip-fields"),
                "invalid number of fields to skip",
            ),
        }
    }

    // Used by tests
    fn new(c: bool, d: bool, u: bool, s: Option<u64>, f: Option<u64>) -> Self {
        Flags {
            show_count:       c,
            supress_unique:   d,
            supress_repeated: u,
            skip_chars:       s,
            skip_fields:      f,
        }
    }
}


fn uniq<R: Read, W: Write>(
    reader: &mut BufReader<R>, writer: &mut W, flags: Flags,
) -> Result<(), io::Error> {
    // Always compared against current_line
    let mut last_line = String::new();
    let mut last_line_count: u64 = 0;

    // Loop for each line read
    loop {
        let mut current_line = String::new();
        let size = reader.read_line(&mut current_line);

        let mut should_exit = false; //
        // Check error or EOF
        match size {
            Err(err) => {
                eprintln!("uniq: input error: {}.", err);
                return Err(err);
            },
            Ok(0) => should_exit = true, // EOF, exit after this loop
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

                if should_show_last_line {
                    writer.write_all(last_line.as_bytes())?;
                    unimplemented!();
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

    // Test utility simplifier, takes input and retrieves uniq's output
    fn test_uniq(input: &str, flags: Flags) -> String {
        let mut reader = BufReader::new(input.as_bytes());

        let mut output = Vec::<u8>::new();
        uniq(&mut reader, &mut output, flags).unwrap();

        std::string::String::from_utf8(output).unwrap()
    }

    // Used by tests with no flags
    fn flags_none() -> Flags { Flags::new(false, false, false, None, None) }

    #[test]
    fn test_uniq_basic_usage() {
        let input = " 1 \n 1 \n 2 \n 3 \n 3 \n 1 ";
        let expected = " 1 \n 2 \n 3 \n 1 ";
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
        assert_eq!("123", test_uniq("123", flags_none())); // Without \n at the end
    }

    #[test]
    fn test_uniq_empty() {
        assert_eq!("", test_uniq("", flags_none()));
    }

    #[test]
    fn test_uniq_count_flag() { let flags = Flags { show_count: true, ..flags_none() }; }

    #[test]
    fn test_uniq_unique_flag() {
        let input = " 1 \n 1 \n 2 \n 3 \n 3 \n 1 ";
        let expected = " 2 \n 1 ";
        let flags = Flags { supress_repeated: true, ..flags_none() };
        assert_eq!(expected, test_uniq(input, flags));
    }

    #[test]
    fn test_uniq_repeated_flag() {
        let input = " 1 \n 1 \n 2 \n 3 \n 3 \n 1 ";
        let expected = " 1 \n 3 \n";
        let flags = Flags { supress_unique: true, ..flags_none() };
        assert_eq!(expected, test_uniq(input, flags));
    }

    #[test]
    fn test_uniq_combined_flags_repeated_unique() {
        let input = " 1 \n 1 \n 2 \n 3 \n 3 \n 1 ";
        let expected = "";
        let flags = Flags { supress_unique: true, supress_repeated: true, ..flags_none() };
        assert_eq!(expected, test_uniq(input, flags));
    }
}
