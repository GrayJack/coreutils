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
                .help("Directory that will be created.")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("mode")
                .help("Set file mode (as in chmod), not a=rwx - umask.")
                .long("mode")
                .short("m")
                .value_name("MODE"),
        )
        .arg(
            Arg::with_name("parents")
                .help("No error if existing, make parent directories as needed.")
                .long("parents")
                .short("p"),
        )
        .arg(
            Arg::with_name("verbose")
                .help("Display a message for each created directory.")
                .long("verbose")
                .short("v"),
        )
}
