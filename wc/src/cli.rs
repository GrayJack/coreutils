use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(Arg::new("FILE").help("File(s) to read.").required(false).multiple_occurrences(true))
        .arg(
            Arg::new("bytes")
                .help("Display the byte counts.")
                .long_help(
                    "Display the byte counts.\n\nIf the current locale does not support multibyte \
                     characters, this is equivalent to the -m option.",
                )
                .long("bytes")
                .short('c'),
        )
        .arg(
            Arg::new("chars")
                .help("Display the character counts.")
                .long_help(
                    "Display the character counts.\n\nIf the current locale does not support \
                     multibyte characters, this is equivalent to the -c option.",
                )
                .long("chars")
                .short('m'),
        )
        .arg(Arg::new("lines").help("Display the newline counts.").long("lines").short('l'))
        .arg(
            Arg::new("max-line-length")
                .help("Display the maximum display width.")
                .long_help(
                    "Display the maximum display width.\n\nWrite the length of the line \
                     containing the most bytes (default) or characters (when -m is provided) to \
                     standard output.\n\nWhen more than one file argument is specified, the \
                     longest input line of all files is reported as the value of the final \
                     \"total\".",
                )
                .long("max-line-length")
                .short('L'),
        )
        .arg(Arg::new("words").help("Display the word counts.").long("words").short('w'))
        .arg(
            Arg::new("pretty")
                .help("Output results in a more human readable format.")
                .long("pretty")
                .short('p'),
        )
}
