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
        .multiple(true)
        .number_of_values(1)
        .empty_values(false);

    app.arg(posix_fmt).arg(command).arg(arguments)
}

#[derive(Debug)]
pub struct TimeOpts {
    pub output_fmt: OutputFormat,
    pub command:    Vec<String>,
}

impl TimeOpts {
    pub fn new() -> TimeOpts {
        let args = create_app().get_matches();
        let command =
            args.value_of("COMMAND").expect("`COMMAND` value cannot be `None`, it is required.");

        TimeOpts {
            output_fmt: if args.is_present("posix") {
                OutputFormat::Posix
            } else {
                OutputFormat::Default
            },
            command:    match args.values_of("arguments") {
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

#[derive(Debug)]
pub enum OutputFormat {
    Default,
    Posix,
}
