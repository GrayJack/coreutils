//! Command line options that are supported by `time`

use crate::cli::create_app;
use crate::output::OutputFormatter;

// Condense CLI args as a struct
#[derive(Debug)]
pub struct TimeOpts {
    /// Formatter to use when printing stats back to CLI
    pub printer: OutputFormatter,
    /// Command as seen on the CLI
    pub command: Vec<String>,
}

impl TimeOpts {
    pub fn from_matches() -> Self {
        Self::new(create_app().get_matches())
    }
    pub fn new(args: clap::ArgMatches) -> Self {
        let command =
            args.value_of("COMMAND").expect("`COMMAND` value cannot be `None`, it is required.");

        TimeOpts {
            printer: if args.is_present("posix") {
                OutputFormatter::Posix
            } else {
                OutputFormatter::Default
            },
            command:    match args.values_of("ARGUMENT") {
                Some(vs) => {
                    let mut cmd = vec![command.to_owned()];
                    cmd.extend(vs.into_iter().map(|item| item.to_owned()));
                    cmd
                },
                None => vec![command.to_owned()],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TimeOpts, OutputFormatter, create_app};

    #[test]
    fn parsing_valid_command_with_args() {
        let args = vec!["test-time", "cmd-to-run", "arg1", "arg2", "arg3"];
        let opts = TimeOpts::new(create_app().get_matches_from(args));

        assert_eq!(4, opts.command.len());
        assert_eq!(vec!["cmd-to-run", "arg1", "arg2", "arg3"], opts.command);
        assert_eq!(OutputFormatter::Default, opts.printer);
    }

    #[test]
    fn parse_valid_command_with_posix_spec() {
        let args = vec!["test-time", "cmd-to-run", "arg1", "arg2", "arg3", "-p"];
        let opts = TimeOpts::new(create_app().get_matches_from(args));

        assert_eq!(OutputFormatter::Posix, opts.printer);
    }
}
