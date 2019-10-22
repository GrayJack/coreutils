use std::{io, io::prelude::*};

#[derive(Debug)]
pub struct Input<'a> {
    msg:     Option<&'a str>,
    err_msg: Option<&'a str>,
}

impl<'a> Default for Input<'a> {
    fn default() -> Input<'a> { Input { msg: None, err_msg: None } }
}

impl<'a> Input<'a> {
    pub fn new() -> Input<'a> { Input::default() }

    pub fn with_msg(&mut self, msg: &'a str) -> &mut Self {
        self.msg = Some(msg);

        self
    }

    pub fn with_err_msg(&mut self, err_msg: &'a str) -> &mut Self {
        self.err_msg = Some(err_msg);

        self
    }

    fn get_input(self) -> Option<String> {
        if let Some(msg) = self.msg {
            print!("{}", msg);
            io::stdout().lock().flush().unwrap();
        }

        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(_) => {},
            Err(err) => {
                if let Some(err_msg) = self.err_msg {
                    eprintln!("{}: {}", err_msg, err);
                } else {
                    eprintln!("{}", err);
                }

                return None;
            },
        };

        Some(line)
    }

    pub fn get(self) -> Option<String> {
        match self.get_input() {
            Some(input) => Some(input.trim().to_owned()),
            None => None,
        }
    }

    pub fn is_affirmative(self) -> bool {
        if let Some(input) = self.get_input() {
            let input = input.trim().to_uppercase();

            input == "Y" || input == "YES" || input == "1"
        } else {
            false
        }
    }
}
