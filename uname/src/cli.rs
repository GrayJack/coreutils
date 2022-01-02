use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("all")
                .help(
                    "Display all information. Behave as though the options -o, -m, -n, -r, -s, \
                     and -v were specified.",
                )
                .long_help(
                    "Display all information.\n\nBehave as though the options -o, -m, -n, -r, -s, \
                     and -v were specified.",
                )
                .long("all")
                .short('a'),
        )
        .arg(
            Arg::new("sysname")
                .help("Display the name of the operating system implementation. (default)")
                .long("sysname")
                .short('s'),
        )
        .arg(
            Arg::new("nodename")
                .help("Display the name of the system to standard output.")
                .long("nodename")
                .short('n'),
        )
        .arg(
            Arg::new("release")
                .help("Display the current release level of the operating system.")
                .long("release")
                .short('r'),
        )
        .arg(
            Arg::new("osversion")
                .help("Display the version level of this release of the operating system.")
                .long("os-version")
                .short('v'),
        )
        .arg(
            Arg::new("machine")
                .help("Display the type of the current hardware platform.")
                .long("machine")
                .short('m'),
        )
        .arg(
            Arg::new("processor")
                .help("Display the machine processor architecture name.")
                .long("processor")
                .short('p'),
        )
        .arg(
            Arg::new("os")
                .help("Display the operating system.")
                .long("operating-system")
                .short('o'),
        )
}
