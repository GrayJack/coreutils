use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

#[allow(clippy::let_and_return)]
pub(crate) fn create_app<'help>() -> App<'help> {
    let app = App::new(crate_name!())
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
        .arg(
            Arg::new("group")
                .help("Display only the effective group ID as number.")
                .long("group")
                .short('g')
                .conflicts_with_all(&["groups", "user", "pretty", "human", "file"]),
        )
        .arg(
            Arg::new("groups")
                .help("Display all group IDs.")
                .long("groups")
                .short('G')
                .conflicts_with_all(&["group", "user", "pretty", "human", "file"]),
        )
        .arg(
            Arg::new("user")
                .help("Display only the effective user ID as number.")
                .long("user")
                .short('u')
                .conflicts_with_all(&["group", "groups", "pretty", "human", "file"]),
        )
        .arg(
            Arg::new("name")
                .help(
                    "Display the name of the user or group ID for the -G, -g and -u options \
                     instead of the number.",
                )
                .long("name")
                .short('n')
                .conflicts_with_all(&["pretty", "human", "file"]),
        )
        .arg(
            Arg::new("real")
                .help("Display the real ID for the -g and -u options instead of the effective ID.")
                .long("real")
                .short('r')
                .conflicts_with_all(&["pretty", "human", "file", "groups"]),
        )
        .arg(
            Arg::new("zero")
                .help("Delimit entries with NULL characters, not whitespace.")
                .long_help(
                    "Delimit entries with NULL characters, not whitespace.\n\nNot permitted in \
                     default format.",
                )
                .long("zero")
                .short('z'),
        )
        .arg(
            Arg::new("pretty")
                .help("Make the output human-readable.")
                .long("pretty")
                .short('p')
                .conflicts_with_all(&["group", "groups", "user", "name", "file"]),
        )
        .arg(
            Arg::new("human")
                .help("Make the output human-readable.")
                .long("human-readable")
                .short('h')
                .conflicts_with_all(&["group", "groups", "user", "name", "file"]),
        )
        .arg(
            Arg::new("file")
                .help("Display the id as a password file entry.")
                .long("file")
                .short('P')
                .conflicts_with_all(&["group", "groups", "user", "name", "pretty", "human"]),
        );

    #[cfg(any(target_os = "freebsd", target_os = "macos"))]
    let app = app.arg(
        Arg::new("audit")
            .help(
                "Display the process audit user ID and other process audit properties, which \
                 requires privilege.",
            )
            .long("audit")
            .short('A'),
    );

    #[cfg(any(target_os = "openbsd"))]
    let app = app.arg(
        Arg::new("rtable")
            .help("Display the routing table of the current process.")
            .long("rtable")
            .short('R'),
    );

    app
}
