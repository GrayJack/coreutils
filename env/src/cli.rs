use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings::TrailingVarArg,
    Arg,
};

// Note that this case needed custom usage string. So any future modification need to pay
// attention to it and modify if necessary
pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .setting(TrailingVarArg)
        .override_usage("env [OPTIONS] [--] [NAME=VALUE]... [COMMAND [ARG]...]")
        .arg(
            Arg::new("OPTIONS")
                .value_name("[NAME=VALUE]... [COMMAND [ARG]...]")
                .help(
                    "Environement variables in the form of NAME=VALUE and the COMMAND to run with \
                     its arguments.",
                )
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("ignore_environment")
                .help("Start with an empty environment")
                .long("ignore-environment")
                .short('i'),
        )
        .arg(
            Arg::new("null")
                .help("End each output line with NUL, not newline.")
                .long("null")
                .short('0'),
        )
        .arg(
            Arg::new("unset")
                .help("Remove variable from the environment")
                .long("unset")
                .short('u')
                .value_name("NAME")
                .multiple_values(true),
        )
}
