use clap::{
    crate_authors, crate_description, crate_name, crate_version, App,
    AppSettings::{ColoredHelp, TrailingVarArg},
    Arg,
};

pub(crate) fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .help_message("Display help information.")
        .version_message("Display version information.")
        .help_short("?")
        .settings(&[ColoredHelp, TrailingVarArg])
        .arg(
            Arg::with_name("COMMAND")
                .help(
                    "Command and arguments to be run which will ignore hangup signals and it's \
                     arguments.",
                )
                .required(true)
                .multiple(true),
        )
}
