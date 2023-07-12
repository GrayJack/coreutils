use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(Arg::new("FILE").help("").multiple_occurrences(true))
        .arg(
            Arg::new("delimiters")
                .help("Loop through characters from LIST instead of using TABs.")
                .long("delimiters")
                .short('d')
                .value_name("LIST"),
        )
        .arg(
            Arg::new("serial")
                .help("Apply paste to lines from each file separately.")
                .long("serial")
                .short('s'),
        )
        .arg(
            Arg::new("zero-terminated")
                .help("Replace newline with NUL as line delimiter.")
                .long("zero-terminated")
                .short('z'),
        )
}
