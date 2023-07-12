use std::{error::Error, fs::File, io};

use clap::ArgMatches;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let flags = PasteFlags::from_matches(&matches);

    let mut readers: Vec<Box<dyn io::BufRead>> = match matches.values_of("FILE") {
        None => vec![Box::new(io::BufReader::new(io::stdin()))],
        Some(filenames) => filenames
            .map(|filename| -> Box<dyn io::BufRead> {
                match filename {
                    "-" => Box::new(io::BufReader::new(io::stdin())),
                    filename => match File::open(filename) {
                        Ok(file) => Box::new(io::BufReader::new(file)),
                        Err(why) => {
                            eprintln!("paste: {}: {}", filename, why);
                            std::process::exit(1);
                        },
                    },
                }
            })
            .collect(),
    };

    std::process::exit(match paste(&mut readers, &flags) {
        Ok(_) => 0,
        Err(why) => {
            eprintln!("paste: {}", why);
            1
        },
    });
}

fn paste(
    readers: &mut Vec<Box<dyn io::BufRead>>, flags: &PasteFlags,
) -> Result<(), Box<dyn Error>> {
    let line_terminator = if flags.zero_terminated { b'\0' } else { b'\n' };

    let mut out_line_no = 0;
    loop {
        let mut flag = false;
        let mut out_buf: Vec<u8> = Vec::new();
        let mut inp_line_no = 0;
        loop {
            let reader_idx = if flags.serial { out_line_no } else { inp_line_no };
            if reader_idx >= readers.len() {
                break;
            }

            let mut inp_buf: Vec<u8> = Vec::new();
            match (*readers[reader_idx]).read_until(line_terminator, &mut inp_buf) {
                Ok(0) => {
                    // EOF
                    if flags.serial {
                        break;
                    }
                },
                Ok(_) => {
                    if pop_if_last(&mut inp_buf, b'\n') {
                        pop_if_last(&mut inp_buf, b'\t');
                    }
                    flag = true;
                },
                Err(why) => {
                    return Err(Box::new(why));
                },
            }

            if inp_line_no != 0 {
                match &flags.delimiters {
                    None => out_buf.push(b'\t'),
                    Some(list) => out_buf.extend([list[(inp_line_no - 1) % list.len()] as u8]),
                };
            }

            out_buf.extend(inp_buf);

            inp_line_no += 1;
        }
        if !flag {
            break;
        }
        let s = match String::from_utf8(out_buf) {
            Ok(string) => string,
            Err(why) => {
                return Err(Box::new(why));
            },
        };
        println!("{}", s);
        out_line_no += 1;
    }
    Ok(())
}

// Pop the last element from `buf` if it equals `tgt`
fn pop_if_last<T>(buf: &mut Vec<T>, tgt: T) -> bool
where
    T: Copy + std::cmp::PartialEq,
{
    if let Some(last) = buf.last().copied() {
        if last == tgt {
            buf.pop();
            return true;
        }
    }
    false
}

struct PasteFlags {
    pub delimiters: Option<Vec<char>>,
    pub serial: bool,
    pub zero_terminated: bool,
}

impl PasteFlags {
    pub fn from_matches(matches: &ArgMatches) -> Self {
        PasteFlags {
            delimiters: match matches.get_one::<String>("delimiters") {
                None => Option::None,
                Some(list) => Option::Some(list.chars().collect()),
            },
            serial: matches.is_present("serial"),
            zero_terminated: matches.is_present("zero-terminated"),
        }
    }
}
