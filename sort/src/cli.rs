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
                .help("The file( operands are processed in command-line order.")
                .long_help(
                    "The file operands are processed in command-line order.\n\nIf file is a \
                     single dash (‘-’) or absent, cat reads from the standard input.",
                )
                .multiple(true),
        )
    // Add args here
}
