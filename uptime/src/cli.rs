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
            Arg::with_name("pretty")
                .help("Display uptime in pretty format.")
                .long("pretty")
                .short("p"),
        )
        .arg(
            Arg::with_name("since")
                .help("System up since.")
                .long_help("System up since.\n\nUses <YYYY-MM-DD hh:mm:ss> format.")
                .long("since")
                .short("s"),
        )
}
