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
            Arg::with_name("DIRECTORY")
                .help("The directory or directories to be removed.")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("ignore-fail-nonempty")
                .help("Ignore each failure that is solely because a directory is non-empty.")
                .long("ignore-fail-on-nonempty")
                .short("I"),
        )
        .arg(
            Arg::with_name("parents")
                .help("Remove DIRECTORY and its ancestors.")
                .long("parents")
                .short("p"),
        )
        .arg(
            Arg::with_name("verbose")
                .help("Output a diagnostic for every directory processed.")
                .long("verbose")
                .short("v"),
        )
}
