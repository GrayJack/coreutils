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
            Arg::with_name("all")
                .help(
                    "Display all information. Behave as though the options -o, -m, -n, -r, -s, \
                     and -v were specified.",
                )
                .long_help(
                    "Display all information.\n\nBehave as though the options -o, -m, -n, -r, -s, \
                     and -v were specified.",
                )
                .long("all")
                .short("a"),
        )
        .arg(
            Arg::with_name("sysname")
                .help("Display the name of the operating system implementation. (default)")
                .long("sysname")
                .short("s"),
        )
        .arg(
            Arg::with_name("nodename")
                .help("Display the name of the system to standard output.")
                .long("nodename")
                .short("n"),
        )
        .arg(
            Arg::with_name("release")
                .help("Display the current release level of the operating system.")
                .long("release")
                .short("r"),
        )
        .arg(
            Arg::with_name("osversion")
                .help("Display the version level of this release of the operating system.")
                .long("os-version")
                .short("v"),
        )
        .arg(
            Arg::with_name("machine")
                .help("Display the type of the current hardware platform.")
                .long("machine")
                .short("m"),
        )
        .arg(
            Arg::with_name("processor")
                .help("Display the machine processor architecture name.")
                .long("processor")
                .short("p"),
        )
        .arg(
            Arg::with_name("os")
                .help("Display the operating system.")
                .long("operating-system")
                .short("o"),
        )
}
