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
            Arg::with_name("STRING")
                .help("The text to be displayed.")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("escape")
                .help("Enable interpretation of backslash escapes.")
                .long("escapes")
                .visible_alias("backslash-escapes")
                .short("e"),
        )
        .arg(
            Arg::with_name("no_newline")
                .help("Do not output the trailing newline.")
                .long("no-newline")
                .short("n"),
        )
}
