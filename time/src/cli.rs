use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings::ColoredHelp, Arg,
};

pub(crate) fn create_app<'a, 'b>() -> App<'a, 'b> {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .help_message("Display help information.")
        .version_message("Display version information.")
        .help_short("?")
        .settings(&[ColoredHelp]);

    let posix_fmt = Arg::with_name("posix")
        .help(
            "Display time output in POSIX specified format as:\n\treal %f\n\tuser %f\n\tsys  \
             %f\nTimer accuracy is arbitrary, but will always be counted in seconds.",
        )
        .short("p")
        .long("--posix")
        .takes_value(false);

    let command = Arg::with_name("COMMAND").help("Command or utility to run.").required(true);

    let arguments = Arg::with_name("ARGUMENT")
        .help("Optional arguments to pass to <COMMAND>.")
        .multiple(true);

    app.args(&[posix_fmt, command, arguments])
}

#[derive(Debug)]
pub struct TimeOpts {
    pub output_fmt: OutputFormatter,
    pub command:    Vec<String>,
}

impl TimeOpts {
    pub fn from_matches() -> Self {
        Self::new(create_app().get_matches())
    }
    pub fn new(args: clap::ArgMatches) -> Self {
        let command =
            args.value_of("COMMAND").expect("`COMMAND` value cannot be `None`, it is required.");

        TimeOpts {
            output_fmt: if args.is_present("posix") {
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

#[derive(Debug, PartialEq)]
pub enum OutputFormatter {
    Default,
    Posix,
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
        assert_eq!(OutputFormatter::Default, opts.output_fmt);
    }

    #[test]
    fn parse_valid_command_with_posix_spec() {
        let args = vec!["test-time", "cmd-to-run", "arg1", "arg2", "arg3", "-p"];
        let opts = TimeOpts::new(create_app().get_matches_from(args));

        assert_eq!(OutputFormatter::Posix, opts.output_fmt);
    }
}
