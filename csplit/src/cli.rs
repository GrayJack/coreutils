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
                .help("File to read, or '-' to read from standard input.")
                .required(true),
        )
        .arg(
            Arg::with_name("PATTERN")
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
                .multiple(true),
        )
        .arg(
            Arg::with_name("prefix")
                .help("Prefix to use for written files.")
                .long("prefix")
                .short("f")
                .takes_value(true)
                .value_name("PREFIX")
                .default_value("xx"),
        )
        .arg(
            Arg::with_name("keep")
                .help("Do not remove output files on error.")
                .long("keep-files")
                .short("k"),
        )
        .arg(
            Arg::with_name("digits")
                .help("Use the given number of digits for output file name.")
                .long("digits")
                .short("n")
                .takes_value(true)
                .value_name("DIGITS")
                .default_value("2"),
        )
        .arg(
            Arg::with_name("silent")
                .help("Do not display counts of output file sizes.")
                .long("silent")
                .visible_alias("quiet")
                .short("s"),
        )
    // .arg(
    //     Arg::with_name("elide-empty")
    //         .help("Remove empty output files.")
    //         .long("elide-empty-files")
    //         .short("z"),
    // )
    // .arg(
    //     Arg::with_name("suppress")
    //         .help("Suppress lines that match a PATTERN.")
    //         .long("suppress-matched")
    //         .short("x"),
    // )
    // .arg(
    //     Arg::with_name("suffix-format")
    //         .help("Format to use for the file suffix")
    //         .long("suffix-format")
    //         .short("b")
    //         .takes_value(true)
    //         .value_name("FORMAT")
    //         .default_value("%02d")
    // )
}
