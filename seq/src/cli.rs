use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};


pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .override_usage("seq [OPTIONS] [FIRST [INCREMENT]] <LAST>")
        .arg(Arg::new("FIRST INCREMENT LAST").required(true).multiple_occurrences(true).hide(true))
        .arg(
            Arg::new("separator")
                .help("Use STRING to separate numbers.")
                .long("separator")
                .short('s')
                .default_value("\n"),
        )
        .arg(
            Arg::new("terminator")
                .help("Terminator of the values.")
                .long("terminator")
                .short('t')
                .default_value("\n"),
        )
        .arg(
            Arg::new("equal-width")
                .help("Equalize the widths of all numbers by padding with zeros as necessary.")
                .long_help(
                    "Equalize the widths of all numbers by padding with zeros as \
                     necessary.\n\nThis option has no effect with the -f option.",
                )
                .long("equal-width")
                .short('w')
                .visible_alias("fixed-width")
                .takes_value(false),
        )
}
