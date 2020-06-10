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
            Arg::with_name("NAME")
                .help("Name of the file(s) to use.")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("multiple")
                .help("Support more than one argument. Treat every argument as a name.")
                .long("multiple")
                .short("a"),
        )
        .arg(
            Arg::with_name("suffix")
                .help("Remove a trailing suffix.")
                .long_help("Remove a trailing suffix.\n\nThis option implies the -a option.")
                .long("suffix")
                .short("s")
                .value_name("SUFFIX"),
        )
        .arg(
            Arg::with_name("zero")
                .help(
                    "Output a zero byte (ASCII NUL) at the end of each line, rather than a \
                     newline.",
                )
                .long("zero")
                .short("z"),
        )
}
