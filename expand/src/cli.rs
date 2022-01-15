use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("FILE")
                .help("File(s) to use, or '-' to use from stdin.")
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("initial")
                .help("Do not convert tabs after non blanks.")
                .long("initial")
                .short('i'),
        )
        .arg(
            Arg::new("tabs")
                .help(
                    "Have tabs N characters apart instead of 8 OR Comma separated LIST of tab \
                     positions.",
                )
                .long_help(
                    "Have tabs N characters apart instead of 8\n\nOR\n\nComma separated LIST of \
                     tab positions.\n\nWhen a LIST of tab positions the last specified position \
                     can be prefixed with '/' to specify a tab size to use after the last \
                     explicitly specified tab stop. Also a prefix of '+' can be used to align \
                     remaining tab stops relative to the last specified tab stop instead of the \
                     first column.",
                )
                .long("tabs")
                .short('t')
                .value_name("N or LIST"),
        )
}
