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
                .help("The file operands are processed in command-line order.")
                .long_help(
                    "The file operands are processed in command-line order.\n\nIf file is a \
                     single dash (‘-’) or absent, cat reads from the standard input.",
                )
                .multiple(true),
        )
        .arg(Arg::with_name("number").help("Number all output lines.").long("number").short("n"))
        .arg(
            Arg::with_name("number_nonblank")
                .help("Number nonempty output lines, overrides -n.")
                .long("number-nonblank")
                .short("b")
                .overrides_with("number"),
        )
        .arg(
            Arg::with_name("show_ends")
                .help("Display $ at end of each line.")
                .long("show-ends")
                .short("E"),
        )
        .arg(
            Arg::with_name("squeeze_blank")
                .help(
                    "Squeeze multiple adjacent empty lines, causing the output to be single \
                     spaced.",
                )
                .long("squeeze-blank")
                .short("s"),
        )
    // Add args here
}
