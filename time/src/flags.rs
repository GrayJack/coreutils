//! Command line options that are supported by `time`

use clap::ArgMatches;

use crate::{cli::create_app, output::OutputFormatter};

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

    pub fn new(args: ArgMatches) -> Self {
        TimeOpts {
            printer: if args.is_present("posix") {
                OutputFormatter::Posix
            } else {
                OutputFormatter::Default
            },
            command: args
                .values_of("COMMAND")
                .expect("`COMMAND` value cannot be `None`, it is required.")
                .map(str::to_owned)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{create_app, OutputFormatter, TimeOpts};

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
        let args = vec!["test-time", "-p", "cmd-to-run", "arg1", "arg2", "arg3"];
        let opts = TimeOpts::new(create_app().get_matches_from(args));

        assert_eq!(OutputFormatter::Posix, opts.printer);
    }
}
