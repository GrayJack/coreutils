use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings::ColoredHelp, Arg,
};

#[allow(clippy::let_and_return)]
pub(crate) fn create_app<'a, 'b>() -> App<'a, 'b> {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .help_message("Display help information.")
        .version_message("Display version information.")
        .help_short("?")
        .settings(&[ColoredHelp])
        .arg(
            Arg::with_name("USER")
                .help("Name of a user.")
                .long_help("Name of a user.\n\nIf not specified assumes the current user"),
        )
        .arg(
            Arg::with_name("group")
                .help("Display only the effective group ID as number.")
                .long("group")
                .short("g")
                .conflicts_with_all(&["groups", "user", "pretty", "human", "file"]),
        )
        .arg(
            Arg::with_name("groups")
                .help("Display all group IDs.")
                .long("groups")
                .short("G")
                .conflicts_with_all(&["group", "user", "pretty", "human", "file"]),
        )
        .arg(
            Arg::with_name("user")
                .help("Display only the effective user ID as number.")
                .long("user")
                .short("u")
                .conflicts_with_all(&["group", "groups", "pretty", "human", "file"]),
        )
        .arg(
            Arg::with_name("name")
                .help(
                    "Display the name of the user or group ID for the -G, -g and -u options \
                     instead of the number.",
                )
                .long("name")
                .short("n")
                .conflicts_with_all(&["pretty", "human", "file"]),
        )
        .arg(
            Arg::with_name("real")
                .help("Display the real ID for the -g and -u options instead of the effective ID.")
                .long("real")
                .short("r")
                .conflicts_with_all(&["pretty", "human", "file", "groups"]),
        )
        .arg(
            Arg::with_name("zero")
                .help("Delimit entries with NULL characters, not whitespace.")
                .long_help(
                    "Delimit entries with NULL characters, not whitespace.\n\nNot permitted in \
                     default format.",
                )
                .long("zero")
                .short("z"),
        )
        .arg(
            Arg::with_name("pretty")
                .help("Make the output human-readable.")
                .long("pretty")
                .short("p")
                .conflicts_with_all(&["group", "groups", "user", "name", "file"]),
        )
        .arg(
            Arg::with_name("human")
                .help("Make the output human-readable.")
                .long("human-readable")
                .short("h")
                .conflicts_with_all(&["group", "groups", "user", "name", "file"]),
        )
        .arg(
            Arg::with_name("file")
                .help("Display the id as a password file entry.")
                .long("file")
                .short("P")
                .conflicts_with_all(&["group", "groups", "user", "name", "pretty", "human"]),
        );

    #[cfg(any(target_os = "freebsd", target_os = "macos"))]
    let app = app.arg(
        Arg::with_name("audit")
            .help(
                "Display the process audit user ID and other process audit properties, which \
                 requires privilege.",
            )
            .long("audit")
            .short("A"),
    );

    #[cfg(any(target_os = "openbsd"))]
    let app = app.arg(
        Arg::with_name("rtable")
            .help("Display the routing table of the current process.")
            .long("rtable")
            .short("R"),
    );

    app
}
