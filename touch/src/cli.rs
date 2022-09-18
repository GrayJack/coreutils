use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("FILE")
                .help(
                    "File(s) to create empty if it does not exist, unless -c or -h is supplied, \
                     or '-' to modify the standard input.",
                )
                .multiple_occurrences(true)
                .required(true),
        )
        .arg(Arg::new("accesstime").help("Change only the access time.").long("atime").short('a'))
        .arg(Arg::new("nocreate").help("Do not create any files.").long("no-create").short('c'))
        .arg(
            Arg::new("modification")
                .help("Change only the modification time.")
                .long("mtime")
                .short('m'),
        )
        .arg(Arg::new("f").help("Ignored. Here for compatibility reasons.").short('f').hide(true))
        .arg(
            Arg::new("reference")
                .help(
                    "Use the access and modifications times from OTHER_FILE instead of the \
                     current time of day.",
                )
                .long("reference")
                .short('r')
                .value_name("OTHER_FILE"),
        )
        .arg(
            Arg::new("no_deref")
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
                .short('h'),
        )
        .arg(
            Arg::new("time")
                .help("Change the specified time acording to WORD value.")
                .long_help(
                    "Change the specified time acording to WORD value.\n\nWhen WORD is access, \
                     atime, or use, the behaviour is equivalent to -a.\nWhen WORD is modify or \
                     mtime, the behaviour is equivalent to -m.",
                )
                .long("time")
                .short('T')
                .value_name("WORD")
                .possible_values(["access", "atime", "modify", "mtime", "use"]),
        )
        .arg(
            Arg::new("date")
                .help(
                    "Parse STRING (date format [Y-M-D h:m:s]) and use it instead of current time.",
                )
                .long("date")
                .short('d')
                .conflicts_with("reference")
                .value_name("STRING"),
        )
        .arg(
            Arg::new("timestamp")
                .help(
                    "Parse STRING (date format [[CC]YY]MMDDhhmm[.ss]) and use it instead of \
                     current time.",
                )
                .short('t')
                .conflicts_with("date")
                .conflicts_with("reference")
                .value_name("STRING"),
        )
}
