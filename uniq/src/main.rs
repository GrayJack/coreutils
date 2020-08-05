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
            eprintln!("uniq: Cannot open '{}' for reading: {}", input_filename, err);
            process::exit(1);
        }))
    };
    let mut reader = BufReader::new(reader);

    let mut writer: Box<dyn io::Write> = if output_filename == "-" {
        Box::new(io::stdout())
    } else {
        Box::new(File::create(output_filename).unwrap_or_else(|err| {
            eprintln!("uniq: Cannot create '{}' for writing: {}", output_filename, err);
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
            if let None = arg {
                None
            } else {
                let number = arg.unwrap().parse::<u64>().unwrap_or_else(|_| {
                    eprintln!("{}", error_msg);
                    process::exit(1);
                });
                Some(number)
            }
        };

        Flags {
            show_count:       matches.is_present("show_count"),
            supress_unique:   matches.is_present("repeated"),
            supress_repeated: matches.is_present("unique"),
            skip_chars:       parse_arg_to_u64(
                matches.value_of("skip-chars"),
                "uniq: a: invalid number of bytes to skip",
            ),
            skip_fields:      parse_arg_to_u64(
                matches.value_of("skip-fields"),
                "uniq: a: invalid number of fields to skip",
            ),
        }
    }
}


fn uniq<R: Read, W: Write>(
    reader: &mut BufReader<R>, writer: &mut W, flags: Flags,
) -> Result<(), io::Error> {
    // Always compared against current_line
    let mut last_line = String::new();

    // Loop for each line read
    loop {
        let mut current_line = String::new();
        let size = reader.read_line(&mut current_line);

        // Check error or EOF
        match size {
            Err(err) => {
                eprintln!("uniq: could not read FAILE: {}", err);
                return Err(err);
            },
            Ok(0) => break, // EOF, stop reading
            Ok(_) => {},
        }

        if current_line != last_line {
            writer.write_all(current_line.as_bytes())?;
        }
        last_line = current_line;
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    // Test utility simplifier, takes input and retrieves uniq's output
    fn test_uniq(input: &str) -> String {
        let mut reader = BufReader::new(input.as_bytes());

        let mut output = Vec::<u8>::new();
        uniq(&mut reader, &mut output).unwrap();

        std::string::String::from_utf8(output).unwrap()
    }


    #[test]
    fn test_uniq_basic() {
        let input = " 1 \n 1 \n 2 \n 3 \n 3 \n";
        let expected = " 1 \n 2 \n 3 \n";
        assert_eq!(expected, test_uniq(input));
    }

    #[test]
    fn test_uniq_without_line_break() {
        assert_eq!("123", test_uniq("123")); // Without \n at the end
    }

    #[test]
    fn test_uniq_empty() {
        assert_eq!("", test_uniq(""));
    }
}
