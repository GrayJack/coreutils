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
        .arg(Arg::with_name("NEWROOT").help("New root directory.").required(true))
        .arg(
            Arg::with_name("COMMAND")
                .help("Shell command to run.")
                .long_help(
                    "Shell command to run.\n\nIf COMMAND is not specified, it defaults to \
                     '$(SHELL) -i'.\nIf $(SHELL) is not set, /bin/sh is used.",
                )
                .multiple(true),
        )
        .arg(
            Arg::with_name("user")
                .help("Specify the user ID or name.")
                .long("user")
                .short("u")
                .takes_value(true)
                .value_name("USER")
                .conflicts_with("userspec"),
        )
        .arg(
            Arg::with_name("group")
                .help("Specify the group ID or name.")
                .long("group")
                .short("g")
                .takes_value(true)
                .value_name("GROUP")
                .conflicts_with("userspec"),
        )
        .arg(
            Arg::with_name("userspec")
                .help("Specify user and group (ID or name) to use.")
                .long_help(
                    "Specify user and group (ID or name) to use.\n\nThe input format is \
                     'USER:GROUP'",
                )
                .long("userspec")
                .short("U")
                .takes_value(true)
                .value_name("USER:GROUP")
                .conflicts_with_all(&["user", "group"]),
        )
        .arg(
            Arg::with_name("groups")
                .help("Specify supplementary groups.")
                .long_help("Specify supplementary groups.\n\nThe GROUPS is a comma separeted list")
                .long("groups")
                .short("G")
                .takes_value(true)
                .value_name("GROUPS"),
        )
}
