use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("SOURCE")
                .help("Source files and destination files.")
                .value_names(&["SOURCE", "DEST"])
                .multiple_occurrences(true)
                .required(true),
        )
        .arg(
            Arg::new("backup")
                .help("Make a backup of each existing destination file.")
                .long("backup")
                .short('b')
                .value_name("TYPE")
                .env("VERSION_CONTROL")
                .default_value("existing")
                .possible_values([
                    "none", "off", "numbered", "t", "existing", "nil", "simple", "never",
                ]),
        )
        .arg(
            Arg::new("force")
                .help("Do not prompt before overwriting.")
                .long_help(
                    "Do not prompt before overwriting.\n\nThis option overrides any previous -n \
                     or -i options.",
                )
                .long("force")
                .short('f')
                .overrides_with_all(&["noClobber", "interactive"]),
        )
        .arg(
            Arg::new("interactive")
                .help("Prompt before overwrite.")
                .long_help(
                    "Prompt before overwrite.\n\nCause mv to write a prompt to standard out \
                     before moving a file that would overwrite an existing file. If the response \
                     from the standard input is affirmative, the move is attempted.\n\nThis \
                     option overrides any previous -f or -n options.",
                )
                .long("interactive")
                .short('i')
                .overrides_with_all(&["noClobber", "force"]),
        )
        .arg(
            Arg::new("noClobber")
                .help("Do not overwrite an existing file.")
                .long_help(
                    "Do not overwrite an existing file.\n\nThis option overrides any previous -f \
                     or -i options.",
                )
                .long("no-clobber")
                .short('n')
                .overrides_with_all(&["force", "interactive"])
                .conflicts_with("backup"),
        )
        .arg(
            Arg::new("stripTrailingSlashes")
                .help("Remove any trailing slashes from each SOURCE argument.")
                .long("strip-trailing-slashes")
                .short('s'),
        )
        .arg(
            Arg::new("suffix")
                .help("Override the usual backup suffix.")
                .long("suffix")
                .short('S')
                .value_name("STRING")
                .env("SIMPLE_BACKUP_SUFFIX")
                .default_value("~"),
        )
        .arg(
            Arg::new("targetDirectory")
                .help("Move all SOURCE arguments into DIRECTORY.")
                .long("target-directory")
                .short('t')
                .value_name("DIRECTORY")
                .conflicts_with("noTargetDirectory"),
        )
        .arg(
            Arg::new("noTargetDirectory")
                .help("Treat DEST as a normal file.")
                .long("no-target-directory")
                .short('T'),
        )
        .arg(
            Arg::new("update")
                .help(
                    "Move only when the SOURCE file is newer than the destination file or when \
                     the destination file is missing.",
                )
                .long("update")
                .short('u'),
        )
        .arg(Arg::new("verbose").help("Explain what is being done.").long("verbose").short('v'))
}
