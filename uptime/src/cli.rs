use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(Arg::new("pretty").help("Display uptime in pretty format.").long("pretty").short('p'))
        .arg(
            Arg::new("since")
                .help("System up since.")
                .long_help("System up since.\n\nUses <YYYY-MM-DD hh:mm:ss> format.")
                .long("since")
                .short('s'),
        )
}
