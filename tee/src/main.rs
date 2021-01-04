use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
};

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();
}

