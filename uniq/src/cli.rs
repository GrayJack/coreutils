use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("INPUT")
                .help("Input file to read, or '-' to read from standard input.")
                .index(1),
        )
        .arg(
            Arg::new("OUTPUT")
                .help("Output file to write, or '-' to write from standard output.")
                .index(2),
        )
        .arg(
            Arg::new("count")
                .help("Prefix lines by the number of occurrences.")
                .short('c')
                .long("count"),
        )
        .arg(
            Arg::new("repeated")
                .help("Only print duplicate lines, one for each group.")
                .short('d')
                .long("repeated"),
        )
        .arg(
            Arg::new("skip-fields")
                .help("Avoid comparing the first N fields.")
                .short('f')
                .long("skip-fields")
                .value_name("N"),
        )
        .arg(
            // We chose "skip-bytes" instead of "skip-chars" in the util internal implementation to
            // avoid confusion.
            Arg::new("skip-bytes")
                .help("Avoid comparing the first N characters.")
                .short('s')
                .long("skip-chars")
                .value_name("N"),
        )
        .arg(Arg::new("unique").help("Only display unique lines.").short('u').long("unique"))
}
