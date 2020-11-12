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
            Arg::with_name("INPUT")
                .help("Input file to read, or '-' to read from standard input.")
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("Output file to write, or '-' to write from standard output.")
                .index(2),
        )
        .arg(
            Arg::with_name("count")
                .help("Prefix lines by the number of occurrences.")
                .short("c")
                .long("count"),
        )
        .arg(
            Arg::with_name("repeated")
                .help("Only print duplicate lines, one for each group.")
                .short("d")
                .long("repeated"),
        )
        .arg(
            Arg::with_name("skip-fields")
                .help("Avoid comparing the first N fields.")
                .short("f")
                .long("skip-fields")
                .value_name("N"),
        )
        .arg(
            // We chose "skip-bytes" instead of "skip-chars" in the util internal implementation to
            // avoid confusion.
            Arg::with_name("skip-bytes")
                .help("Avoid comparing the first N characters.")
                .short("s")
                .long("skip-chars")
                .value_name("N"),
        )
        .arg(Arg::with_name("unique").help("Only display unique lines.").short("u").long("unique"))
}
