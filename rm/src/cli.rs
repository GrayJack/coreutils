use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("FILE")
                .help("File(s) to be deleted.")
                .long_help(
                    "File(s) to be deleted.\n\nTo remove a file whose name starts with a '-', for \
                     example '-foo', use 'rm -- -foo'",
                )
                .required(true)
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("force")
                .help(
                    "Ignore nonexistent files, never prompt regardless of the file's permissions.",
                )
                .long_help(
                    "Ignore nonexistent files, never prompt regardless of the file's \
                     permissions.\n\nThis option overrides any previous -i or -I.",
                )
                .long("force")
                .short('f')
                .overrides_with_all(&["interactive", "interactiveBatch"]),
        )
        .arg(
            Arg::new("interactive")
                .help("Prompt before every removal.")
                .long_help(
                    "Prompt before every removal.\n\nThis option overrides any previous -f or -I.",
                )
                .long("interactive")
                .short('i')
                .overrides_with_all(&["force", "interactiveBatch"]),
        )
        .arg(
            Arg::new("interactiveBatch")
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
                .short('I')
                .overrides_with_all(&["force", "interactive"]),
        )
        .arg(
            Arg::new("noPreserveRoot")
                .help("Do not treat '/' specially.")
                .long("no-preserve-root")
                .short('n')
                .conflicts_with("preserveRoot"),
        )
        .arg(
            Arg::new("preserveRoot")
                .help("Do not remove '/'. (default)")
                .long("preserve-root")
                .short('p')
                .conflicts_with("noPreserveRoot"),
        )
        .arg(
            Arg::new("recursive")
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
                .short('r'),
        )
        .arg(
            Arg::new("recursive_compat")
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
                .short('R'),
        )
        .arg(Arg::new("directories").help("Remove empty directories.").long("dir").short('d'))
        .arg(Arg::new("verbose").help("Explain what is being done.").long("verbose").short('v'))
}
