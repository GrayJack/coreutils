use clap::{
    crate_authors, crate_description, crate_name, crate_version, App,
    AppSettings::{ColoredHelp, TrailingVarArg},
    Arg,
};

// Note that this case needed custom usage string. So any future modification need to pay
// attention to it and modify if necessary
pub(crate) fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .help_message("Display help information.")
        .version_message("Display version information.")
        .help_short("?")
        .settings(&[ColoredHelp, TrailingVarArg])
        .usage("env [FLAGS] [OPTIONS] [--] [NAME=VALUE]... [COMMAND [ARG]...]")
        .arg(
            Arg::with_name("OPTIONS")
                .value_name("[NAME=VALUE]... [COMMAND [ARG]...]")
                .help(
                    "Environement variables in the form of NAME=VALUE and the COMMAND to run with \
                     its arguments.",
                )
                .multiple(true),
        )
        .arg(
            Arg::with_name("ignore_environment")
                .help("Start with an empty environment")
                .long("ignore-environment")
                .short("i"),
        )
        .arg(
            Arg::with_name("null")
                .help("End each output line with NUL, not newline.")
                .long("null")
                .short("0"),
        )
        .arg(
            Arg::with_name("unset")
                .help("Remove variable from the environment")
                .long("unset")
                .short("u")
                .value_name("NAME")
                .multiple(true),
        )
}
