use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(Arg::new("NAME").help("Name of the FIFO.").required(true))
        .arg(
            Arg::new("mode")
                .help("Set file permission bits to MODE, not a=rw - umask")
                .long("mode")
                .short('m')
                .default_value("644"),
        )
}
