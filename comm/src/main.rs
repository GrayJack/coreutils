use std::{
    fs::File,
    io::{prelude::*, BufReader, ErrorKind}, process::exit,
};

use clap::ArgMatches;
mod cli;

struct Flags {
    pub one : bool,
    pub two : bool,
    pub three : bool
}


fn main() {
    let matches = cli::create_app().get_matches();

    let flags = Flags::user_gave(&matches);

    let file1: String = match matches.values_of("file_1") {
        Some(m) => m.map(String::from).collect(),
        None => "".to_string(),
    };

    let file2: String = match matches.values_of("file_2") {
        Some(m) => m.map(String::from).collect(),
        None => "".to_string(),
    };

    let mut content_1 : Vec<String> = Vec::new();
    let mut content_2 : Vec<String> = Vec::new();

    
    match File::open(&file1) {
        Ok(file) => {
            for line in BufReader::new(file).lines() {
                let line = match line {
                    Ok(line) => line,
                    Err(error) => {
                        eprintln!("comm: {}", error);
                        exit(1);
                    },
                };
                content_1.push(line);
            }
        },
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    eprintln!("comm: {}: No such file or directory", file1)
                },
                ErrorKind::PermissionDenied => {
                    eprintln!("comm: {}: Permission denied", file1)
                },
                _ => eprintln!("comm: {}: Unknown error", file1),
            }
        },
    };

    match File::open(&file2) {
        Ok(file) => {
            for line in BufReader::new(file).lines() {
                let line = match line {
                    Ok(line) => line,
                    Err(error) => {
                        eprintln!("comm: {}", error);
                        exit(1);
                    },
                };
                content_2.push(line);
            }
        },
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    eprintln!("comm: {}: No such file or directory", file2)
                },
                ErrorKind::PermissionDenied => {
                    eprintln!("comm: {}: Permission denied", file2)
                },
                _ => eprintln!("comm: {}: Unknown error", file2),
            }
        },
    };

    helper(flags, &content_1, &content_2);

    exit(0);
}

impl Flags {
    pub fn user_gave (matches: &ArgMatches) -> Self {
        Flags {
            one :  matches.is_present("1"),
            two : matches.is_present("2"),
            three : matches.is_present("3")
        }
    }
}

fn is_sorted<I>(data: I) -> bool where
    I: IntoIterator,
    I::Item: Ord,
{
    let mut it = data.into_iter();
    match it.next() {
        None => true,
        Some(first) => it.scan(first, |state, next| {
            let cmp = *state <= next;
            *state = next;
            Some(cmp)
        }).all(|b| b),
    }
}

fn helper(flags: Flags, content_1: &Vec<String>, content_2: &Vec<String>) {
    assert!(is_sorted(content_1), "The file 1, is not in sorted order.");
    assert!(is_sorted(content_2), "The file 2, is not in sorted order.");

    //0 -> unique to 1
    //1 -> unique to 2
    //2 -> common

    let mut order : Vec<(String, i32)> = Vec::new();
    
    let n : usize  = content_1.len();
    let m : usize  = content_2.len();

    let mut i : usize = 0;
    let mut j : usize = 0;

    while i < n && j < m {
        let here1 = &content_1[i];
        let here2 = &content_2[j];

        if content_1[i] == content_2[j] {
            order.push((here1.to_string(), 2));
            i += 1;
            j += 1;

        } else if content_1[i] < content_2[j] {
            order.push((here1.to_string(), 0));
            i += 1;
        } else {
            order.push((here2.to_string(), 1));
            j += 1;
        }
    }

    while i < n {
        let here = &content_1[i];
        order.push((here.to_string(), 0));
        i += 1;
    }

    while j < m {
        let here = &content_2[j];
        order.push((here.to_string(), 1));
        j += 1;
    }

    let tab : String = "       ".to_string();

    if flags.one {

        let mut i : usize = 0;
        let n : usize = order.len();

        while i < n {
            if order[i].1 == 0 {
                i += 1;
                continue;
            }

            if order[i].1 == 1 {
                println!("{}", order[i].0);
            } else {
                println!("{} {}", tab, order[i].0);
            }
         
            i += 1;
         
        }
    } else if flags.two {

        let mut i : usize = 0;
        let n : usize = order.len();

        while i < n {
            if order[i].1 == 1 {
                i += 1;
                continue;
            }

            if order[i].1 == 0 {
                println!("{}", order[i].0);
            } else {
                println!("{} {}", tab, order[i].0);
            }
         
            i += 1;
         
        }

    } else if flags.three {

        let mut i : usize = 0;
        let n : usize = order.len();

        while i < n {
            if order[i].1 == 2 {
                i += 1;
                continue;
            }

            if order[i].1 == 0 {
                println!("{}", order[i].0);
            } else {
                println!("{} {}", tab, order[i].0);
            }
         
            i += 1;
         
        }

    } else {

        let mut i : usize = 0;
        let n : usize = order.len();

        while i < n {
            
            if order[i].1 == 0 {
                println!("{} {}", order[i].0, tab);
            } else if order[i].1 == 1 {
                println!("{} {}", tab, order[i].0 );
            } else {
                println!("{} {} {} ", tab, tab, order[i].0);
            }
         
            i += 1;
         
        }
    }
}