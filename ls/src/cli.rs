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
                .help("File(s) to list")
                .required(true)
                .multiple(true)
                .default_value("."),
        )
        .arg(
            Arg::with_name("all")
                .help("Do not ignore entries starting with .")
                .short("a")
                .long("all"),
        )
        .arg(
            Arg::with_name("classify")
                .help("Append indicator (one of */=>@|) to entries")
                .short("F")
                .long("classify"),
        )
        .arg(
            Arg::with_name("comma_separate")
                .help("Fill width with a comma separated list of entries")
                .short("m"),
        )
        .arg(
            Arg::with_name("indicator")
                .help("Write a <slash> ( '/' ) after each filename if that file is a directory")
                .short("p"),
        )
        .arg(
            Arg::with_name("last_accessed")
                .help("Use time of last access instead of last modification of the file for sorting -t or writing -l")
                .short("u"),
        )
        .arg(Arg::with_name("list").help("Use a long listing format").short("l"))
        .arg(Arg::with_name("no_owner").help("Like -l, but do not list owner").short("g"))
        .arg(
            Arg::with_name("numeric_uid_gid")
                .help("Like -l, but list numeric user and group IDs")
                .short("n")
                .long("numeric-uid-gid"),
        )
        .arg(
            Arg::with_name("reverse")
                .help("Reverse order while sorting")
                .short("r")
                .long("reverse"),
        )
        .arg(
            Arg::with_name("size")
                .help("Print the allocated size of each file, in blocks")
                .short("s")
                .long("size"),
        )
        .arg(
            Arg::with_name("sort_size")
                .help("Sort by first file size, largest first")
                .short("S"),
        )
        .arg(Arg::with_name("time").help("Sort by modification time, newest first").short("t"))
}
