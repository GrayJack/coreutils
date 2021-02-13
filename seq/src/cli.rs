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
        .usage("seq [FLAGS] [OPTIONS] [FIRST [INCREMENT]] <LAST>")
        .arg(Arg::with_name("FIRST INCREMENT LAST").required(true).hidden(true))
        .arg(
            Arg::with_name("SEPARATOR")
                .short("s")
                .long("separator")
                .help("Use STRING to separate numbers.")
                .hide_default_value(true)
                .default_value("\n"),
        )
        .arg(
            Arg::with_name("WIDTH")
                .short("w")
                .long("equal-width")
                .visible_alias("fixed-width")
                .help("Equalize the widths of all numbers by padding with zeros as necessary.")
                .long_help(
                    "Equalize the widths of all numbers by padding with zeros as \
                     necessary.\n\nThis option has no effect with the -f option.",
                )
                .takes_value(false),
        )
}
