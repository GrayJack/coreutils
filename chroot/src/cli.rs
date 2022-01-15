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
        .arg(Arg::new("NEWROOT").help("New root directory.").required(true))
        .arg(
            Arg::new("COMMAND")
                .help("Shell command to run.")
                .long_help(
                    "Shell command to run.\n\nIf COMMAND is not specified, it defaults to \
                     '$(SHELL) -i'.\nIf $(SHELL) is not set, /bin/sh is used.",
                )
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("user")
                .help("Specify the user ID or name.")
                .long("user")
                .short('u')
                .value_name("USER")
                .conflicts_with("userspec"),
        )
        .arg(
            Arg::new("group")
                .help("Specify the group ID or name.")
                .long("group")
                .short('g')
                .value_name("GROUP")
                .conflicts_with("userspec"),
        )
        .arg(
            Arg::new("userspec")
                .help("Specify user and group (ID or name) to use.")
                .long_help(
                    "Specify user and group (ID or name) to use.\n\nThe input format is \
                     'USER:GROUP'",
                )
                .long("userspec")
                .short('U')
                .value_name("USER:GROUP")
                .conflicts_with_all(&["user", "group"]),
        )
        .arg(
            Arg::new("groups")
                .help("Specify supplementary groups.")
                .long_help("Specify supplementary groups.\n\nThe GROUPS is a comma separeted list")
                .long("groups")
                .short('G')
                .value_name("GROUPS"),
        )
}
