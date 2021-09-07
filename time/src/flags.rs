//! Command line options that are supported by `time`

use std::{fs::OpenOptions, io, path::PathBuf};

use clap::ArgMatches;

use crate::{cli::create_app, output::FormatterKind, output::OutputFormatter};

// Condense CLI args as a struct
#[derive(Debug)]
pub struct TimeOpts {
    /// Formatter to use when printing stats back to CLI
    pub printer: OutputFormatter,
    /// Command as seen on the CLI
    pub command: Vec<String>,
    /// Where the output should be written to
    pub destination: Option<PathBuf>,
    /// Should the destination be appended to?
    pub append: bool,
}

impl TimeOpts {
    pub fn from_matches() -> Self {
        Self::new(create_app().get_matches())
    }

    pub fn get_output_stream(&self) -> Result<Box<dyn io::Write>, io::Error> {
        match &self.destination {
            Some(dest) => {
                let file = if self.append {
                    OpenOptions::new().append(true).create(true).open(dest)
                } else {
                    OpenOptions::new().write(true).truncate(true).create(true).open(dest)
                };
                file.map(|f| Box::new(f) as Box<dyn io::Write>)
            },
            None => Ok(Box::new(io::stderr())),
        }
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
            destination: match args.value_of("output_file") {
                Some("-") => None,
                Some(dest) => Some(PathBuf::from(dest)),
                None => None,
            },
            append: args.is_present("append_mode"),
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
