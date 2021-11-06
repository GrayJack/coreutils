mod cli;

use std::{process, result::Result, str::FromStr};

use clap::Values;
use coreutils_core::strings::StringEscapeDecoder;

const ARG_UNDERFLOW_ERROR: i32 = 64;
const ARG_UNPARSABLE_ERROR: i32 = 63;

fn main() {
    let matches = cli::create_app().get_matches();

    let format_string = matches.value_of("FORMAT").unwrap();
    let args = matches.values_of("ARGUMENTS");

    match parse_format(format_string, args) {
        Ok(result) => println!("{}", result),
        Err(error_code) => process::exit(error_code),
    }
}

fn parse_format(format_string: &str, args: Option<Values>) -> Result<String, i32> {
    let format_string: Vec<char> = StringEscapeDecoder::from(format_string).collect();
    let mut args = args;

    let mut parts = vec![];
    let mut chars = vec![];
    let mut fmt_index: usize = 0;

    while fmt_index < format_string.len() {
        match format_string[fmt_index] {
            '%' => {
                // Format sequence
                fmt_index += 1;

                let part: String = chars.into_iter().collect();
                parts.push(part);
                chars = vec![];

                match format_string[fmt_index] {
                    'c' => {
                        let c = get_arg::<char>(&mut args)?;
                        parts.push(c.to_string());
                    },
                    'd' => {
                        let int = get_arg::<i32>(&mut args)?;
                        parts.push(format!("{}", int));
                    },
                    'e' | 'g' => {
                        // Scientific notation of floats
                        let double = get_arg::<f64>(&mut args)?;
                        parts.push(format!("{:e}", double));
                    },
                    'E' | 'G' => {
                        // Scientific notation of floats
                        let double = get_arg::<f64>(&mut args)?;
                        parts.push(format!("{:E}", double));
                    },
                    'f' => {
                        let float = get_arg::<f32>(&mut args)?;
                        parts.push(format!("{}", float));
                    },
                    'h' => {
                        fmt_index += 1;

                        match format_string[fmt_index] {
                            'i' => {
                                let short = get_arg::<i16>(&mut args)?;
                                parts.push(format!("{}", short));
                            },
                            'u' => {
                                let short = get_arg::<u16>(&mut args)?;
                                parts.push(format!("{}", short));
                            },
                            _ => return Err(ARG_UNPARSABLE_ERROR),
                        }
                    },
                    'i' => {
                        let int = get_arg::<u32>(&mut args)?;
                        parts.push(format!("{}", int));
                    },
                    'l' => {
                        fmt_index += 1;

                        match format_string[fmt_index] {
                            'd' | 'i' => {
                                let long = get_arg::<i32>(&mut args)?;
                                parts.push(format!("{}", long));
                            },
                            'u' => {
                                let short = get_arg::<u32>(&mut args)?;
                                parts.push(format!("{}", short));
                            },
                            'f' => {
                                let double = get_arg::<f64>(&mut args)?;
                                parts.push(format!("{}", double));
                            },
                            'l' => {
                                fmt_index += 1;

                                match format_string[fmt_index] {
                                    'd' | 'i' => {
                                        let long = get_arg::<i64>(&mut args)?;
                                        parts.push(format!("{}", long));
                                    },
                                    'u' => {
                                        let short = get_arg::<u64>(&mut args)?;
                                        parts.push(format!("{}", short));
                                    },
                                    _ => return Err(ARG_UNPARSABLE_ERROR),
                                }
                            },
                            _ => {
                                let long = get_arg::<i32>(&mut args)?;
                                parts.push(format!("{}", long));
                            },
                        }
                    },
                    'o' => {
                        let long = get_arg::<i32>(&mut args)?;
                        parts.push(format!("{:o}", long));
                    },
                    's' => {
                        let string = get_arg::<String>(&mut args)?;
                        parts.push(string);
                    },
                    'u' => {
                        let int = get_arg::<u32>(&mut args)?;
                        parts.push(format!("{}", int));
                    },
                    'x' => {
                        let long = get_arg::<i32>(&mut args)?;
                        parts.push(format!("{:x}", long));
                    },
                    'X' => {
                        let long = get_arg::<i32>(&mut args)?;
                        parts.push(format!("{:X}", long));
                    },
                    'n' => {
                        // Just pop an arg off the list
                        let _ = get_arg::<String>(&mut args)?;
                    },
                    '%' => parts.push("%".to_owned()),
                    _ => {
                        return Err(ARG_UNPARSABLE_ERROR);
                    },
                }
            },
            _ => chars.push(format_string[fmt_index]),
        }
        fmt_index += 1;
    }

    let part: String = chars.into_iter().collect();
    parts.push(part);

    println!();
    let result: String = parts.into_iter().collect();

    Ok(result)
}

fn to<T: FromStr>(string: &str) -> Result<T, i32> {
    if let Ok(to_type) = string.parse() {
        return Ok(to_type);
    }

    Err(ARG_UNPARSABLE_ERROR)
}

fn get_arg<T: FromStr>(args: &mut Option<Values>) -> Result<T, i32> {
    if let Some(args) = args {
        if let Some(arg) = args.next() {
            return to(arg);
        }
    }

    Err(ARG_UNDERFLOW_ERROR)
}
