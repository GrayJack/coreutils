use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("FILE").help("File(s) to use, or '-' to use from standard input.").multiple_occurrences(true),
        )
        .arg(
            Arg::new("body_numbering")
                .help("Use STYLE for numbering body lines.")
                .long_help(
                    "Use STYLE for numbering body lines.\n\nSTYLE is one of:\n\ta      number all \
                    lines\n\tt      number only nonempty lines\n\tn      number no lines\n\tpBRE   \
                    number only lines that contain a match for the basic regular expression, BRE",
                )
                .long("body-numbering")
                .short('b')
                .value_name("STYLE")
                .default_value("t"),
        )
        .arg(
            Arg::new("section_delimiter")
                .help("Use CC for logical page delimiters.")
                .long_help(
                    "Use CC for logical page delimiters.\n\nCC are two delimiter characters used \
                     to construct logical page delimiters; a missing second character implies ':'.",
                )
                .long("section-delimiter")
                .short('d')
                .value_name("CC")
                .default_value(r"\:"),
        )
        .arg(
            Arg::new("footer_numbering")
                .help("Use STYLE for numbering footer lines.")
                .long_help(
                    "Use STYLE for numbering footer lines.\n\nSTYLE is one of:\n\ta      number \
                    all lines\n\tt      number only nonempty lines\n\tn      number no lines\n\t\
                    pBRE   number only lines that contain a match for the basic regular \
                    expression, BRE",
                )
                .long("footer-numbering")
                .short('f')
                .value_name("STYLE")
                .default_value("n"),
        )
        .arg(
            Arg::new("header_numbering")
                .help("Use STYLE for numbering header lines.")
                .long_help(
                    "Use STYLE for numbering header lines.\n\nSTYLE is one of:\n\ta      number \
                    all lines\n\tt      number only nonempty lines\n\tn      number no lines\n\t\
                    pBRE   number only lines that contain a match for the basic regular \
                    expression, BRE",
                )
                .long("header-numbering")
                .short('h')
                .value_name("STYLE")
                .default_value("n"),
        )
        .arg(
            Arg::new("line_increment")
                .help("Line number increment at each line.")
                .long("line-increment")
                .short('i')
                .value_name("NUMBER")
                .default_value("1"),
        )
        .arg(
            Arg::new("join_blank_lines")
                .help("Group of NUMBER empty lines counted as one.")
                .long_help(
                    "Group of NUMBER empty lines counted as one.\n\nIf numbering of all lines is \
                     specified for the current logical section using the corresponding '-b a', \
                     '-f a' or '-H' a option, specify the number of adjacent blank lines to be \
                     considered as one. For example, '-l 2' results in only the second adjacent \
                     blank line being numbered.",
                )
                .long("join-blank-lines")
                .short('l')
                .value_name("NUMBER")
                .default_value("1"),
        )
        .arg(
            Arg::new("number_format")
                .help("Insert line numbers according to FORMAT.")
                .long_help(
                    "Insert line numbers according to FORMAT.\n\nFORMAT is one of:\n\tln     left \
                    justified, no leading zeros\n\trn     right justified, no leading zeros\n\t\
                    rz     right justified, leading zeros",
                )
                .long("number-format")
                .short('n')
                .value_name("FORMAT")
                .default_value("rn")
                .possible_values(&["ln", "rn", "rz"]),
        )
        .arg(
            Arg::new("no_renumber")
                .help("Do not reset line numbers for each section.")
                .long("no-renumber")
                .short('p'),
        )
        .arg(
            Arg::new("number_separator")
                .help(
                    "Add STRING after (possible) line number. If not specified, defaults to <TAB>.",
                )
                .long("number-separator")
                .short('s')
                .value_name("STRING"),
        )
        .arg(
            Arg::new("starting_line_number")
                .long("starting-line-number")
                .help("First line number for each section.")
                .short('v')
                .value_name("NUMBER")
                .default_value("1"),
        )
        .arg(
            Arg::new("number_width")
                .help("Use NUMBER columns for line numbers.")
                .long("number-width")
                .short('w')
                .value_name("NUMBER")
                .default_value("6"),
        )
}
