use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("DIRECTORY")
                .help("The directory or directories to be removed.")
                .required(true)
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("ignore-fail-nonempty")
                .help("Ignore each failure that is solely because a directory is non-empty.")
                .long("ignore-fail-on-nonempty")
                .short('I'),
        )
        .arg(
            Arg::new("parents")
                .help("Remove DIRECTORY and its ancestors.")
                .long("parents")
                .short('p'),
        )
        .arg(
            Arg::new("verbose")
                .help("Output a diagnostic for every directory processed.")
                .long("verbose")
                .short('v'),
        )
}
