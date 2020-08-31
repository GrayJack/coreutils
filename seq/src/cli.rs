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
        .arg(Arg::with_name("FIRST INCREMENT LAST").multiple(true))
        .arg(
            Arg::with_name("SEPERATOR")
                .short("s")
                .long("separator")
                .help("use STRING to separate numbers (default: \\n)")
                .hide_default_value(true)
                .default_value("\n"),
        )
        .arg(
            Arg::with_name("WIDTH")
                .short("w")
                .long("equal-width")
                .help("equalize width by padding with leading zeroes")
                .takes_value(false),
        )
}
