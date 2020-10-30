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
                .help("File(s) to be sorted, merged or checked.")
                .long_help(
                    "File(s) to be sorted, merged or checked.\n\nIf FILE is ‘-’ or absent, sort \
                     reads from the standard input.",
                )
                .multiple(true),
        )
        .arg(
            Arg::with_name("merge_only")
                .help("Merge files.")
                .long_help("Merge files\n\nThe input FILE(s) are assumed to be already sorted.")
                .short("m")
                .long("merge"),
        )
        .arg(
            Arg::with_name("output")
                .value_name("FILE")
                .help("Write result to FILE instead of standard output.")
                .long_help(
                    "Write result to FILE instead of standard output.\n\nThis FILE can be the \
                     same one as one of the input FILE(s).",
                )
                .short("o"),
        )
    // Add args here
}
