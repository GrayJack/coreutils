use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("OPERAND")
                .help(
                    "An operand with a leading plus sign ('+') signals a user-defined format \
                     string which specifies the format in which to display the date and time.",
                )
                .long_help(
                    "An operand with a leading plus sign ('+') signals a user-defined format \
                     string which specifies the format in which to display the date and \
                     time.\n\nThe format string may contain any of the conversion specifications \
                     described in the time crate documentation, as well as any arbitrary \
                     text.\n\nA newline ('\\n') character is always output after the characters \
                     specified by the format string.\n\nIf specified with other arguments that \
                     also sets an OUTPUT_FMT/FMT, this one will be used.",
                )
                .value_name("+OUTPUT_FMT"),
        )
        .arg(Arg::new("DATE").help("Display time described by STRING, not 'now'.").long_help(
            "Display time described by STRING, not 'now'.\n\nShould be of value \
             [[[[[CC]YY]MM]DD]hh]mm[.SS].",
        ))
        .arg(
            Arg::new("utc")
                .help("Display Coordinated Universal Time (UTC).")
                .long("utc")
                .visible_alias("universal")
                .short('u'),
        )
        .arg(
            Arg::new("rfc2822")
                .help("Use RFC 2822 date and time as OUTPUT_FMT.")
                .long_help(
                    "Use RFC 2822 date and time output format.\n\nThis is equivalent to using \
                     \"%a, %d %b %Y %T %z\" as OUTPUT_FMT while LC_TIME is set to the \"C\" \
                     locale.\n\nIf '+' operand is specified, this option will be ignored.\n\nIf \
                     more than one argument that changes OUTPUT_FMT is set, the last ones is used.",
                )
                .long("rfc2822")
                .visible_alias("rfc-2822")
                .short('R')
                .overrides_with_all(&["iso8601", "rfc3339"]),
        )
        .arg(
            Arg::new("reference")
                .help(
                    "Display the date and time of the last modification of filename OR the date \
                     and time represented by seconds since UNIX Epoch.",
                )
                .long_help(
                    "Display the date and time of the last modification of \
                     filename.\n\nOR\n\nDisplay the date and time represented by seconds, where \
                     seconds is the number of seconds since the Epoch (00:00:00 UTC, January 1, \
                     1970; see time(3)).",
                )
                .long("reference")
                .visible_alias("read")
                .short('r')
                .value_name("FILENAME | SECONDS"),
        )
        .arg(
            Arg::new("iso8601")
                .help("Use ISO 8601 date and time as OUTPUT_FMT.")
                .long_help(
                    "Use ISO 8601 date and time as OUTPUT_FMT.\n\nFMT may not be omitted, \
                     different of other versions of this util.\n\nThe date and time is formatted \
                     to the specified precision. When FMT is 'hours' (or the more precise \
                     'minutes' or 'seconds'), the ISO 8601 format includes the timezone.\n\nIf \
                     '+' operand is specified, this option will be ignored.\n\nIf more than one \
                     argument that changes OUTPUT_FMT is set, the last ones is used.",
                )
                .long("iso8601")
                .visible_alias("iso-8601")
                .short('I')
                .value_name("FMT")
                .possible_values([
                    "date", "hour", "hours", "minute", "minutes", "second", "seconds",
                ])
                .overrides_with_all(&["rfc2822", "rfc3339"]),
        )
        .arg(
            Arg::new("rfc3339")
                .help("Use RFC 3339 date and time as OUTPUT_FMT.")
                .long_help(
                    "Use RFC 3339 date and time as OUTPUT_FMT.\n\nFMT may not be omitted, \
                     different of other versions of this util.\n\nThe date and time is formatted \
                     to the specified precision. When FMT is 'hours' (or the more precise \
                     'minutes', 'seconds' or 'nanoseconds'), the RFC 3339 format includes the \
                     timezone.\n\nIf '+' operand is specified, this option will be ignored.\n\nIf \
                     more than one argument that changes OUTPUT_FMT is set, the last ones is used.",
                )
                .long("rfc3339")
                .visible_alias("rfc-3339")
                .short('S')
                .value_name("FMT")
                .possible_values([
                    "date",
                    "hour",
                    "hours",
                    "minute",
                    "minutes",
                    "second",
                    "seconds",
                    "nanosecond",
                    "nanoseconds",
                    "ns",
                ])
                .overrides_with_all(&["iso8601", "rfc2822"]),
        );

    if cfg!(not(target_os = "haiku")) {
        app.arg(
            Arg::new("set")
                .help("Try to set the date.")
                .long("set")
                .short('s')
                .overrides_with("no_set"),
        )
        .arg(
            Arg::new("no_set")
                .help("Do not try to set the date.")
                .long("no-set")
                .visible_alias("convert")
                .short('j')
                .overrides_with("set"),
        )
    } else {
        app
    }

    // There is no good way to implement it right now
    // .arg(
    //     Arg::new("format")
    //         .help(
    //             "Use INPUT_FMT as the format string to parse the DATE provided rather
    // than \              using the default [[[[[CC]YY]MM]DD]hh]mm[.SS] format.",
    //         )
    //         .long("format")
    //         .short('f')
    //         .value_name("INPUT_FMT"),
    // )
}
