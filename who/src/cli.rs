use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings::ColoredHelp, Arg,
};

pub(crate) fn create_app<'a, 'b>() -> App<'a, 'b> {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .help_message("Display help information.")
        .version_message("Display version information.")
        .help_short("?")
        .settings(&[ColoredHelp])
        .arg(
            Arg::with_name("FILE")
                .help("A file to use to get the user information.")
                .required(false),
        )
        .arg(
            Arg::with_name("all")
                .help(
                    #[cfg(target_os = "openbsd")]
                    "Display all possible information. (Equivalent of -iT)",
                    #[cfg(not(target_os = "openbsd"))]
                    "Display all possible information. (Equivalent of -bdlprTtuv)",
                )
                .long("all")
                .short("a"),
        )
        .arg(
            Arg::with_name("heading")
                .help("Display line of collumn headings.")
                .long("heading")
                .short("H"),
        )
        .arg(
            Arg::with_name("associated_stdin")
                .help("Only display information about the current terminal.")
                .long("associated-stdin")
                .short("m"),
        )
        .arg(
            Arg::with_name("count")
                .help("Display all login names and number of users logged on.")
                .long("count")
                .short("q"),
        )
        .arg(
            Arg::with_name("short")
                .help("Display only name, line, and time. (default)")
                .long("short")
                .short("s"),
        )
        .arg(
            Arg::with_name("message")
                .help(
                    "Display a character after the user name indicating the state of the terminal \
                     line.",
                )
                .long_help(
                    "Display a character after the user name indicating the state of the terminal \
                     line.\n\nPossible states:\n\t- '+' if the terminal is writable;\n\t- '-' if \
                     it is not;\n\t- '?' if a bad line is encountered.",
                )
                .long("message")
                .short("T"),
        )
        .arg(
            Arg::with_name("idle")
                .help("Display the idle time for each user.")
                .long("idle")
                .short("i"),
        );

    #[cfg(not(target_os = "openbsd"))]
    let app = app
        .arg(
            Arg::with_name("boot")
                .help("Display the time of the last system boot.")
                .long("boot")
                .short("b"),
        )
        .arg(Arg::with_name("dead").help("Display dead processes.").long("dead").short("d"))
        .arg(
            Arg::with_name("login")
                .help("Display system login processes.")
                .long("login")
                .short("l"),
        )
        .arg(
            Arg::with_name("process")
                .help("Display active processes spawned by init.")
                .long("process")
                .short("p"),
        )
        .arg(
            Arg::with_name("runlevel")
                .help("Display current run level.")
                .long("runlevel")
                .short("r"),
        )
        .arg(
            Arg::with_name("time")
                .help("Display last system clock change.")
                .long("time")
                .short("t"),
        )
        .arg(
            Arg::with_name("users")
                .help("Display the idle time for each user.")
                .long("users")
                .short("u"),
        );

    app
}
