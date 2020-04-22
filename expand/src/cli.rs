use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings::ColoredHelp, Arg,
};

pub(crate) fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .help_message("Display help information.")
        .version_message("Display version information.")
        .help_short("?")
        .settings(&[ColoredHelp])
        .arg(
            Arg::with_name("FILE").help("File(s) to use, or '-' to use from stdin.").multiple(true),
        )
        .arg(
            Arg::with_name("initial")
                .help("Do not convert tabs after non blanks.")
                .long("initial")
                .short("i"),
        )
        .arg(
            Arg::with_name("tabs")
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
                .short("t")
                .value_name("N or LIST"),
        )
}
