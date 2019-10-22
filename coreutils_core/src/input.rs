use std::{io, io::prelude::*};

#[derive(Debug)]
pub struct Input {
    input:   Option<String>,
    msg:     Option<String>,
    err_msg: Option<String>,
}

impl Default for Input {
    fn default() -> Input { Input { input: None, msg: None, err_msg: None } }
}

impl Input {
    pub fn new() -> Self { Input::default() }

    pub fn with_msg(&mut self, msg: &str) -> &Self {
        self.msg = Some(msg.to_string());

        self
    }

    pub fn with_err_msg(&mut self, err_msg: &str) -> &Self {
        self.err_msg = Some(err_msg.to_string());

        self
    }

    fn get_input(&mut self) {
        if let Some(msg) = &self.msg {
            print!("{}", msg);
            io::stdout().lock().flush().unwrap();
        }

        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(_) => {},
            Err(err) => {
                if let Some(err_msg) = &self.err_msg {
                    eprintln!("{}: {}", err_msg, err);
                } else {
                    eprintln!("{}", err);
                }

                self.input = None;
            },
        };

        self.input = Some(line);
    }

    pub fn get(&mut self) -> Option<&str> {
        self.get_input();

        match self.input {
            Some(input) => Some(input.trim()),
            None => None,
        }
    }

    pub fn is_affirmative(&mut self) -> bool {
        self.get_input();

        if let Some(input) = &self.input {
            let input = input.trim().to_uppercase();

            input == "Y" || input == "YES" || input == "1"
        } else {
            false
        }
    }
}
