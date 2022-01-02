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
                .help("File to read, or '-' to read from standard input.")
                .required(true),
        )
        .arg(
            Arg::new("PATTERN")
                .help("Patterns to use when splitting file.")
                .long_help("Patterns to use when splitting file.\n\n\
                    PATTERN can be any of:\n\t\
                    INTEGER            copy lines up to line number\n\t\
                    /REGEXP/[OFFSET]   copy lines up to line matching REGEXP\n\t\
                    %REGEXP%[OFFSET]   skip lines up to line matching REGEXP\n\t\
                    {INTEGER}          repeat preceeding pattern INTEGER times\n\t\
                    {*}                repeat preceeding pattern indefinitely\n\n\
                    If an OFFSET is given it should be an integer, either positive or negative. \
                    An offset without sign is assumed to be positive.")
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("prefix")
                .help("Prefix to use for written files.")
                .long("prefix")
                .short('f')
                .value_name("PREFIX")
                .default_value("xx"),
        )
        .arg(
            Arg::new("keep")
                .help("Do not remove output files on error.")
                .long("keep-files")
                .short('k'),
        )
        .arg(
            Arg::new("digits")
                .help("Use the given number of digits for output file name.")
                .long("digits")
                .short('n')
                .value_name("DIGITS")
                .default_value("2"),
        )
        .arg(
            Arg::new("silent")
                .help("Do not display counts of output file sizes.")
                .long("silent")
                .visible_alias("quiet")
                .short('s'),
        )
    // .arg(
    //     Arg::new("elide-empty")
    //         .help("Remove empty output files.")
    //         .long("elide-empty-files")
    //         .short('z'),
    // )
    // .arg(
    //     Arg::new("suppress")
    //         .help("Suppress lines that match a PATTERN.")
    //         .long("suppress-matched")
    //         .short('x'),
    // )
    // .arg(
    //     Arg::new("suffix-format")
    //         .help("Format to use for the file suffix")
    //         .long("suffix-format")
    //         .short('b')
    //         .value_name("FORMAT")
    //         .default_value("%02d")
    // )
}
