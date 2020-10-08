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
            Arg::with_name("FILE")
                .help(
                    "File(s) to create empty if it does not exist, unless -c or -h is supplied, \
                     or '-' to modify the standard input.",
                )
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("accesstime")
                .help("Change only the access time.")
                .long("atime")
                .short("a"),
        )
        .arg(
            Arg::with_name("nocreate")
                .help("Do not create any files.")
                .long("no-create")
                .short("c"),
        )
        .arg(
            Arg::with_name("modification")
                .help("Change only the modification time.")
                .long("mtime")
                .short("m"),
        )
        .arg(
            Arg::with_name("f")
                .help("Ignored. Here for compatibility reasons.")
                .short("f")
                .hidden(true),
        )
        .arg(
            Arg::with_name("reference")
                .help(
                    "Use the access and modifications times from OTHER_FILE instead of the \
                     current time of day.",
                )
                .long("reference")
                .short("r")
                .value_name("OTHER_FILE"),
        )
        .arg(
            Arg::with_name("no_deref")
                .help(
                    "If the file is a symbolic link, change the times of the link itself rather \
                     than the file that the link points to.",
                )
                .long_help(
                    "If the file is a symbolic link, change the times of the link itself rather \
                     than the file that the link points to.\n\nNote that -h implies -c and thus \
                     will not create any new files.",
                )
                .long("no-deref")
                .visible_alias("no-dereference")
                .short("h"),
        )
        .arg(
            Arg::with_name("time")
                .help("Change the specified time acording to WORD value.")
                .long_help(
                    "Change the specified time acording to WORD value.\n\nWhen WORD is access, \
                     atime, or use, the behaviour is equivalent to -a.\nWhen WORD is modify or \
                     mtime, the behaviour is equivalent to -m.",
                )
                .long("time")
                .short("T")
                .value_name("WORD")
                .possible_values(&["access", "atime", "modify", "mtime", "use"]),
        )
        .arg(
            Arg::with_name("date")
                .help(
                    "Parse STRING (date format [Y-M-D h:m:s]) and use it instead of current time.",
                )
                .long("date")
                .short("d")
                .conflicts_with("reference")
                .value_name("STRING"),
        )
        .arg(
            Arg::with_name("timestamp")
                .help(
                    "Parse STRING (date format [[CC]YY]MMDDhhmm[.ss]) and use it instead of \
                     current time.",
                )
                .short("t")
                .conflicts_with("date")
                .conflicts_with("reference")
                .value_name("STRING"),
        )
}
