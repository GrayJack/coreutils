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
                .help("File(s) to read, or '-' to read from standard input.")
                .multiple(true)
                .required(false),
        )
        .arg(
            Arg::with_name("bytes")
                .help("Select only these bytes.")
                .long_help("Select only these bytes.\n\nThe LIST specifies byte positions.")
                .long("bytes")
                .short("b")
                .value_name("LIST")
                .conflicts_with_all(&["chars", "fields"]),
        )
        .arg(
            Arg::with_name("chars")
                .help("Select only these characters.")
                .long_help(
                    "Select only these characters.\n\nThe LIST specifies character positions.",
                )
                .long("characters")
                .visible_alias("chars")
                .short("c")
                .value_name("LIST")
                .conflicts_with_all(&["bytes", "fields"]),
        )
        .arg(
            Arg::with_name("input-delimiter")
                .help("Use DELIM instead of TAB for field delimiter.")
                .long("delimiter")
                .short("d")
                .requires("field")
                .value_name("DELIM"),
        )
        .arg(
            Arg::with_name("fields")
                .help(
                    "Select only these fields. Will display any line that contains no delimiter \
                     character, unless the -s option is specified.",
                )
                .long_help(
                    "Select only these fields. Will display any line that contains no delimiter \
                     character, unless the -s option is specified.\n\nThe LIST specifies fields, \
                     separated in the input by the field delimiter character (see the 'delimeter' \
                     option).\n\nOutput fields are separated by a single occurrence of the field \
                     delimiter character.",
                )
                .long("fields")
                .short("f")
                .conflicts_with_all(&["bytes", "chars"])
                .value_name("LIST"),
        )
        .arg(
            Arg::with_name("complement")
                .help("Complement the set of selected bytes, characters, or fields.")
                .long("complement")
                .short("C"),
        )
        .arg(
            Arg::with_name("only-delimited")
                .help("Do not display lines not containing delimiters.")
                .long("only-delimited")
                .short("s")
                .requires("fields"),
        )
        .arg(
            Arg::with_name("output-delimiter")
                .help("Use STRING as the output delimiter. Defaults to use the input delimiter.")
                .long("output-delimiter")
                .short("D")
                .value_name("STRING"),
        )
        .arg(
            Arg::with_name("zero-terminated")
                .help("Line delimiter is NUL. Default is to use newline.")
                .long("zero-terminated")
                .short("z"),
        )
}
