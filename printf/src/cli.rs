use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(Arg::new("FORMAT").help("Format string.").required(true))
        .arg(
            Arg::new("ARGUMENTS")
                .help("Arguments to be formatting with the format string.")
                .multiple_occurrences(true),
        )
}
