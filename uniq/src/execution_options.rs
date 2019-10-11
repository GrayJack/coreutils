use std::process;

pub enum Counters {
    All,
    UniqueOnly,
    DuplicateOnly
}

pub struct ExecutionOptions {
    count_kind: Counters,
    ignore_case: bool,
    read_stdin: bool,
    field_skip: u32,
    char_skip: u32,
    write_stdout: bool,
    input_file_path: Option<String>,
    output_file_path: Option<String>,
}

fn parse_num_or_panic(num_s: &str, message: String) -> u32 {
    match num_s.parse() {
        Ok(n) => n,
        Err(_err) => {
            eprintln!("{}", message);
            process::exit(1);
        }
    }
}

/// Accept a match object from clap, and return a struct representing current
/// CLI options
///
/// # Arguments
/// * `matches` - A `clap::ArgMatches` object to transform into a struct
///   representing the parameters of execution
pub fn read_clap_matches(arg_matches: clap::ArgMatches) -> ExecutionOptions {
    let read_stdin = !arg_matches.is_present("input_file");
    let write_stdout = !arg_matches.is_present("output_file");

    ExecutionOptions {
        count_kind:
            if arg_matches.is_present("duplicate-only") {
                Counters::DuplicateOnly
            } else if arg_matches.is_present("unique-only") {
                Counters::UniqueOnly
            } else {
                Counters::All
            },
        ignore_case: arg_matches.is_present("ignore-case"),
        field_skip: if arg_matches.is_present("num") {
            let num_s = arg_matches.value_of("num").unwrap();
            parse_num_or_panic(num_s,
                               format!("uniq: Illegal field skip value: {}", num_s))
        } else {
            0
        },
        char_skip: if arg_matches.is_present("chars") {
            let num_s = arg_matches.value_of("chars").unwrap();
            parse_num_or_panic(num_s,
                               format!("uniq: Illegal character skip value: {}", num_s))
        } else {
            0
        },
        read_stdin,
        write_stdout,
        input_file_path: if read_stdin {
            None
        } else {
            match arg_matches.value_of("input_file") {
                Some(ref path) =>
                    if path == &"-" {
                        None
                    }  else {
                        Some(path.to_string())
                    }
                None => None,
            }
        },
        output_file_path: if write_stdout {
            None
        } else {
            match arg_matches.value_of("output_file") {
                Some(ref path) => Some(path.to_string()),
                None => None,
            }
        },
    }
}
