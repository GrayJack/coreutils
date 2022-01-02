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
                .help("The file(s) operands are processed in command-line order.")
                .long_help(
                    "The file operands are processed in command-line order.\n\nIf file is a \
                     single dash (`-`) or absent, cat reads from the standard input.",
                )
                .multiple_occurrences(true),
        )
        .arg(Arg::new("number").help("Number all output lines.").long("number").short('n'))
        .arg(
            Arg::new("number_nonblank")
                .help("Number nonempty output lines, overrides -n.")
                .long("number-nonblank")
                .short('b')
                .overrides_with("number"),
        )
        .arg(
            Arg::new("show_ends")
                .help("Display $ at end of each line.")
                .long("show-ends")
                .short('E'),
        )
        .arg(
            Arg::new("squeeze_blank")
                .help(
                    "Squeeze multiple adjacent empty lines, causing the output to be single \
                     spaced.",
                )
                .long("squeeze-blank")
                .short('s'),
        )
    // Add args here
}
