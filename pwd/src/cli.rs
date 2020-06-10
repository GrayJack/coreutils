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
            Arg::with_name("logical")
                .help("Display the logical current working directory.")
                .long_help(
                    "Display the logical current working directory.\n\nUse PWD from environment, \
                     even if it contains symlinks.",
                )
                .long("logical")
                .short("l")
                .overrides_with("physical"),
        )
        .arg(
            Arg::with_name("physical")
                .help("Display the physical current working directory. Avoid all symlinks.")
                .long_help(
                    "Display the physical current working directory. Avoid all symlinks (All \
                     symbolic links resolved).",
                )
                .long("physical")
                .short("p")
                .overrides_with("logical"),
        )
}
