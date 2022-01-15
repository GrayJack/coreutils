use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("INPUT_FILES")
                .value_name("INPUT_FILES")
                .help("File(s) to be sorted, merged or checked.")
                .long_help(
                    "File(s) to be sorted, merged or checked.\n\nIf FILE is ‘-’ or absent, sort \
                     reads from the standard input.",
                )
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("merge_only")
                .help("Merge files.")
                .long_help("Merge files\n\nThe input FILE(s) are assumed to be already sorted.")
                .short('m')
                .long("merge"),
        )
        .arg(
            Arg::new("output")
                .value_name("FILE")
                .help("Write result to FILE instead of standard output.")
                .long_help(
                    "Write result to FILE instead of standard output.\n\nThis FILE can be the \
                     same one as one of the input FILE(s).",
                )
                .long("output")
                .short('o'),
        )
    // Add args here
}
