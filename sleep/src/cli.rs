use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("NUMBER")
                .help(
                    "Number to set the sleep. If more than one, will sleep the sum of the numbers.",
                )
                .required(true)
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("suffix")
                .help("A optional parameter to set the time measurement to be used.")
                .long("suffix")
                .short('s')
                .value_name("SUFFIX")
                .default_value("sec"),
        )
}
