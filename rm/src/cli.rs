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
        .arg(
            Arg::with_name("FILE")
                .help("File(s) to be deleted.")
                .long_help(
                    "File(s) to be deleted.\n\nTo remove a file whose name starts with a '-', for \
                     example '-foo', use 'rm -- -foo'",
                )
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("force")
                .help(
                    "Ignore nonexistent files, never prompt regardless of the file's permissions.",
                )
                .long_help(
                    "Ignore nonexistent files, never prompt regardless of the file's \
                     permissions.\n\nThis option overrides any previous -i or -I.",
                )
                .long("force")
                .short("f")
                .overrides_with_all(&["interactive", "interactiveBatch"]),
        )
        .arg(
            Arg::with_name("interactive")
                .help("Prompt before every removal.")
                .long_help(
                    "Prompt before every removal.\n\nThis option overrides any previous -f or -I.",
                )
                .long("interactive")
                .short("i")
                .overrides_with_all(&["force", "interactiveBatch"]),
        )
        .arg(
            Arg::with_name("interactiveBatch")
                .help(
                    "Prompt once before removing more than three files, or when removing \
                     recursively.",
                )
                .long_help(
                    "Prompt once before removing more than three files, or when removing \
                     recursively.\n\nLess intrusive than -i, while still giving protection \
                     against most mistakes.\n\nThis option overrides any previous -f or -i.",
                )
                .long("interactive-batch")
                .short("I")
                .overrides_with_all(&["force", "interactive"]),
        )
        .arg(
            Arg::with_name("noPreserveRoot")
                .help("Do not treat '/' specially.")
                .long("no-preserve-root")
                .short("n")
                .conflicts_with("preserveRoot"),
        )
        .arg(
            Arg::with_name("preserveRoot")
                .help("Do not remove '/'. (default)")
                .long("preserve-root")
                .short("p")
                .conflicts_with("noPreserveRoot"),
        )
        .arg(
            Arg::with_name("recursive")
                .help("Remove directories and their contents recursively.")
                .long_help(
                    "Remove directories and their contents recursively.\n\nThis option implies \
                     the -d option.\n\nIf the -i option is specified, the user is prompted for \
                     confirmation before each directory's contents are processed (as well as \
                     before the attempt is made to remove the directory).\n\nIf the user does not \
                     respond affirmatively, the file hierarchy rooted in that directory is \
                     skipped.",
                )
                .long("recursive")
                .short("r"),
        )
        .arg(
            Arg::with_name("recursive_compat")
                .help("Remove directories and their contents recursively.")
                .long_help(
                    "Remove directories and their contents recursively.\n\nThis option implies \
                     the -d option.\n\nIf the -i option is specified, the user is prompted for \
                     confirmation before each directory's contents are processed (as well as \
                     before the attempt is made to remove the directory).\n\nIf the user does not \
                     respond affirmatively, the file hierarchy rooted in that directory is \
                     skipped.",
                )
                .long("recursive-compat")
                .short("R"),
        )
        .arg(Arg::with_name("directories").help("Remove empty directories.").long("dir").short("d"))
        .arg(
            Arg::with_name("verbose")
                .help("Explain what is being done.")
                .long("verbose")
                .short("v"),
        )
}
