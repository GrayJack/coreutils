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
            Arg::with_name("USER")
                .help("Name of a user.")
                .long_help("Name of a user.\n\nIf not specified assumes the caller user"),
        )
        .arg(
            Arg::with_name("id")
                .help("Give the group id with the group name.")
                .long("id")
                .short("i"),
        )
}
