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
                .help("File(s) to write to.")
                .long_help(
                    "File(s) to write to.\n\nIf file is a single dash (`-`), it shall refer to a \
                     file named `-`.",
                )
                .multiple_occurrences(true),
        )
        .arg(Arg::new("append").help("Append the output to the files.").long("append").short('a'))
        .arg(
            Arg::new("ignore_interrupts")
                .help("Ignore interrupt signals.")
                .long("ignore-interrupts")
                .short('i'),
        )
}
