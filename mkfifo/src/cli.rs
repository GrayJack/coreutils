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
        .arg(Arg::with_name("NAME").help("Name of the FIFO.").required(true))
        .arg(
            Arg::with_name("mode")
                .help("Set file permission bits to MODE, not a=rw - umask")
                .long("mode")
                .short("m")
                .default_value("644"),
        )
}
