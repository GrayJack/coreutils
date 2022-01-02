use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("STRING")
                .help("The text to be displayed.")
                .required(true)
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("escape")
                .help("Enable interpretation of backslash escapes.")
                .long("escapes")
                .visible_alias("backslash-escapes")
                .short('e'),
        )
        .arg(
            Arg::new("no_newline")
                .help("Do not output the trailing newline.")
                .long("no-newline")
                .short('n'),
        )
}
