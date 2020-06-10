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
        .arg(Arg::with_name("FILE").help("File(s) to read.").required(false).multiple(true))
        .arg(
            Arg::with_name("bytes")
                .help("Display the byte counts.")
                .long_help(
                    "Display the byte counts.\n\nIf the current locale does not support multibyte \
                     characters, this is equivalent to the -m option.",
                )
                .long("bytes")
                .short("c"),
        )
        .arg(
            Arg::with_name("chars")
                .help("Display the character counts.")
                .long_help(
                    "Display the character counts.\n\nIf the current locale does not support \
                     multibyte characters, this is equivalent to the -c option.",
                )
                .long("chars")
                .short("m"),
        )
        .arg(Arg::with_name("lines").help("Display the newline counts.").long("lines").short("l"))
        .arg(
            Arg::with_name("max-line-length")
                .help("Display the maximum display width.")
                .long_help(
                    "Display the maximum display width.\n\nWrite the length of the line \
                     containing the most bytes (default) or characters (when -m is provided) to \
                     standard output.\n\nWhen more than one file argument is specified, the \
                     longest input line of all files is reported as the value of the final \
                     \"total\".",
                )
                .long("max-line-length")
                .short("L"),
        )
        .arg(Arg::with_name("words").help("Display the word counts.").long("words").short("w"))
        .arg(
            Arg::with_name("pretty")
                .help("Output results in a more human readable format.")
                .long("pretty")
                .short("p"),
        )
}
