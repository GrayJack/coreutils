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
                .help("File(s) to read, or '-' to read from standard input.")
                .multiple_occurrences(true)
                .required(false),
        )
        .arg(
            Arg::new("bytes")
                .help("Select only these bytes.")
                .long_help("Select only these bytes.\n\nThe LIST specifies byte positions.")
                .long("bytes")
                .short('b')
                .value_name("LIST")
                .conflicts_with_all(&["chars", "fields"]),
        )
        .arg(
            Arg::new("chars")
                .help("Select only these characters.")
                .long_help(
                    "Select only these characters.\n\nThe LIST specifies character positions.",
                )
                .long("characters")
                .visible_alias("chars")
                .short('c')
                .value_name("LIST")
                .conflicts_with_all(&["bytes", "fields"]),
        )
        .arg(
            Arg::new("input-delimiter")
                .help("Use DELIM instead of TAB for field delimiter.")
                .long("delimiter")
                .short('d')
                .requires("fields")
                .value_name("DELIM"),
        )
        .arg(
            Arg::new("fields")
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
                .short('f')
                .conflicts_with_all(&["bytes", "chars"])
                .value_name("LIST"),
        )
        .arg(
            Arg::new("complement")
                .help("Complement the set of selected bytes, characters, or fields.")
                .long("complement")
                .short('C'),
        )
        .arg(
            Arg::new("only-delimited")
                .help("Do not display lines not containing delimiters.")
                .long("only-delimited")
                .short('s')
                .requires("fields"),
        )
        .arg(
            Arg::new("output-delimiter")
                .help("Use STRING as the output delimiter. Defaults to use the input delimiter.")
                .long("output-delimiter")
                .short('D')
                .value_name("STRING"),
        )
        .arg(
            Arg::new("zero-terminated")
                .help("Line delimiter is NUL. Default is to use newline.")
                .long("zero-terminated")
                .short('z'),
        )
}
