//! The Input module handles getting user input from the command line

use std::{io, io::prelude::*};

/// The Input struct handles issuing messages and getting responses from the user.
///
/// ## Example
/// To ask the user whether they want to make a change or not, and validate their response
/// into a [`bool`], the [`Input`] struct can be used like so:
/// ```rust,ignore
/// let answer: bool = Input::new()
///     .with_msg("Do you want to make this change?")
///     .with_err_msg("Error! Failure to read!")
///     .is_affirmative();
///
/// assert_eq!(answer, true);
/// ```
///
/// One could also get the response directly from the user like so:
/// ```rust,ignore
/// let answer: String = Input::new()
///     .with_msg("Do you want to make this change?")
///     .with_err_msg("Error! Failure to read!")
///     .get();
///
/// assert_eq!(answer, String::from("Yes, I do"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Input<'a> {
    msg:     Option<&'a str>,
    err_msg: Option<&'a str>,
}

impl<'a> Input<'a> {
    /// Initializes a new Input builder.
    pub const fn new() -> Self { Input { msg: None, err_msg: None } }

    /// Specifies the message to display to the user.
    pub fn with_msg(&mut self, msg: &'a str) -> &mut Self {
        self.msg = Some(msg);

        self
    }

    /// Specifies the error message to display to the user.
    /// **NOTE:** This error message prepends the actual error message produced.
    pub fn with_err_msg(&mut self, err_msg: &'a str) -> &mut Self {
        self.err_msg = Some(err_msg);

        self
    }

    fn get_input(&self) -> Option<String> {
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

    /// Gets the input value the user entered as a [`String`].
    pub fn get(&self) -> Option<String> {
        match self.get_input() {
            Some(input) => Some(input.trim().to_string()),
            None => None,
        }
    }

    /// Verifies whether the user input is considered an 'affirmative' answer.
    pub fn is_affirmative(&self) -> bool {
        if let Some(input) = self.get_input() {
            let input = input.trim().to_uppercase();

            input == "Y" || input == "YES" || input == "1"
        } else {
            false
        }
    }
}
