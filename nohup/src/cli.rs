use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings::TrailingVarArg,
    Arg,
};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .setting(TrailingVarArg)
        .arg(
            Arg::new("COMMAND")
                .help(
                    "Command and arguments to be run which will ignore hangup signals and it's \
                     arguments.",
                )
                .required(true)
                .multiple_occurrences(true),
        )
}
