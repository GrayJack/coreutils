use clap::{
    crate_authors, crate_description, crate_name, crate_version, App,
    AppSettings::{AllowNegativeNumbers, TrailingVarArg},
    Arg,
};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .setting(AllowNegativeNumbers | TrailingVarArg)
        .arg(
            Arg::new("COMMAND")
                .help("Command to be run with modified niceness and it's arguments.")
                .multiple_occurrences(true)
                .required(true),
        )
        .arg(
            Arg::new("adjustment")
                .help(
                    "A positive or negative decimal integer used to modify the system scheduling \
                     priority of utility.",
                )
                .long("adjustment")
                .short('n')
                .value_name("N")
                .default_value("10"),
        )
}
