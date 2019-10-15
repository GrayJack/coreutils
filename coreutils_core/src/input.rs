use std::{io, io::prelude::*, process};

#[derive(Debug)]
pub struct Input(String);

impl Input {
    pub fn new() -> Self {
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(_) => {},
            Err(err) => {
                eprintln!("rm: cannot read input: {}", err);
                process::exit(1);
            },
        };

        Input(line)
    }

    pub fn with_msg(msg: &str) -> Self {
        print!("{}", msg);

        if let Err(err) = io::stdout().lock().flush() {
            eprintln!("rm: could not flush stdout: {}", err);
            process::exit(1);
        }

        Self::new()
    }

    pub fn is_affirmative(&self) -> bool {
        let input = self.0.trim().to_uppercase();

        input == "Y" || input == "YES" || input == "1"
    }
}
