use clap::{
    crate_authors, crate_description, crate_name, crate_version, App,
    AppSettings::{AllowNegativeNumbers, ColoredHelp, TrailingVarArg},
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
        .settings(&[ColoredHelp, AllowNegativeNumbers, TrailingVarArg])
        .arg(
            Arg::with_name("COMMAND")
                .help("Command to be run with modified niceness and it's arguments.")
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("adjustment")
                .help(
                    "A positive or negative decimal integer used to modify the system scheduling \
                     priority of utility.",
                )
                .long("adjustment")
                .short("n")
                .value_name("N")
                .default_value("10"),
        )
}
