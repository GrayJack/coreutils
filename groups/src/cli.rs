use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("USER")
                .help("Name of a user.")
                .long_help("Name of a user.\n\nIf not specified assumes the current user"),
        )
        .arg(Arg::new("id").help("Give the group id with the group name.").long("id").short('i'))
}
