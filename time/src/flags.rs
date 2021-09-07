//! Command line options that are supported by `time`

use clap::ArgMatches;

use crate::{cli::create_app, output::FormatterKind, output::OutputFormatter};

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
            printer: {
                let kind = if args.is_present("posix") {
                    FormatterKind::Posix
                } else if args.is_present("use_csh_fmt") {
                    FormatterKind::CSH
                } else if args.is_present("use_tcsh_fmt") {
                    FormatterKind::TCSH
                } else if args.is_present("format_string") {
                    FormatterKind::FmtString(
                        args.value_of("format_string").expect("Empty format string").to_owned(),
                    )
                } else {
                    FormatterKind::Default
                };
                OutputFormatter {
                    kind,
                    human_readable: args.is_present("human_readable")
                }
            },
            command: args
                .values_of("COMMAND")
                .expect("`COMMAND` to run is required")
                .map(str::to_owned)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{create_app, FormatterKind, TimeOpts};

    #[test]
    fn parsing_valid_command_with_args() {
        let args = vec!["test-time", "cmd-to-run", "arg1", "arg2", "arg3"];
        let opts = TimeOpts::new(create_app().get_matches_from(args));

        assert_eq!(4, opts.command.len());
        assert_eq!(vec!["cmd-to-run", "arg1", "arg2", "arg3"], opts.command);
        assert_eq!(FormatterKind::Default, opts.printer.kind);
    }

    #[test]
    fn parse_valid_command_with_posix_spec() {
        let args = vec!["test-time", "-p", "cmd-to-run", "arg1", "arg2", "arg3"];
        let opts = TimeOpts::new(create_app().get_matches_from(args));

        assert_eq!(FormatterKind::Posix, opts.printer.kind);
    }
}
