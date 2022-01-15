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
                .help("Directory that will be created.")
                .required(true)
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("mode")
                .help("Set file mode (as in chmod), not a=rwx - umask.")
                .long("mode")
                .short('m')
                .value_name("MODE"),
        )
        .arg(
            Arg::new("parents")
                .help("No error if existing, make parent directories as needed.")
                .long("parents")
                .short('p'),
        )
        .arg(
            Arg::new("verbose")
                .help("Display a message for each created directory.")
                .long("verbose")
                .short('v'),
        )
}
