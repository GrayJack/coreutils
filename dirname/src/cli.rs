use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("NAME")
                .help("Name of the file to use.")
                .required(true)
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("zero")
                .help(
                    "Output a zero byte (ASCII NUL) at the end of each line, rather than a \
                     newline.",
                )
                .long("zero")
                .short('z'),
        )
}
