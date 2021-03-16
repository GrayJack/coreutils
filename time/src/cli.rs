use clap::{
    crate_authors, crate_description, crate_name, crate_version, App,
    AppSettings::{ColoredHelp, TrailingVarArg},
    Arg,
};

pub(crate) fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .help_message("Display help information.")
        .version_message("Display version information.")
        .help_short("?")
        .settings(&[ColoredHelp, TrailingVarArg])
        .arg(
            Arg::with_name("COMMAND")
                .help("Command to run and it's arguments.")
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("posix")
                .help(
                    "Display time output in POSIX specified format as:\n\treal %f\n\tuser \
                     %f\n\tsys  %f\nTimer accuracy is arbitrary, but will always be counted in \
                     seconds.",
                )
                .long("posix")
                .short("p"),
        )
}
