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
            Arg::with_name("NUMBER")
                .help(
                    "Number to set the sleep. If more than one, will sleep the sum of the numbers.",
                )
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("suffix")
                .help("A optional parameter to set the time measurement to be used.")
                .long("suffix")
                .short("s")
                .value_name("SUFFIX")
                .default_value("sec"),
        )
}
