use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(Arg::new("FILE").help("File(s) to use.").multiple_occurrences(true))
        .arg(
            Arg::new("bytes")
                .help("Display the byte counts.")
                .long("bytes")
                .short('c')
                .value_name("N")
                .conflicts_with("lines"),
        )
        .arg(
            Arg::new("lines")
                .help("Display the newline counts.")
                .long("lines")
                .short('n')
                .value_name("N")
                .default_value("10"),
        )
}
