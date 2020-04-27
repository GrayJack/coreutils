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
            Arg::with_name("TEMPLATE")
                .help("Template to use when creating the temporary file/directory.")
                .long_help(
                    "Template to use when creating the temporary file/directory.\n\nThe template \
                     may be any file name with some number of X's appended to it.",
                ),
        )
        .arg(
            Arg::with_name("directory")
                .help("Make a directory instead of a file.")
                .long("directory")
                .short("d"),
        )
        .arg(
            Arg::with_name("quiet")
                .help("Fail silently if an error occurs.")
                .long_help(
                    "Fail silently if an error occurs. This is useful if a script does not want \
                     error output to go to standard error.",
                )
                .long("quiet")
                .short("q"),
        )
        .arg(
            Arg::with_name("t")
                .help(
                    "Create the file/directory in the directory specified by the TMPDIR \
                     environment variable if set, otherwise /tmp/.",
                )
                .long_help(
                    "Create the file/directory in the directory specified by the TMPDIR \
                     environment variable if set, otherwise /tmp/.\n\nIf TEMPLATE doesnot end \
                     with X, generate a template using TEMPLATE as a prefix.\n\nIf TEMPLATE does \
                     end with X, use it directly as a template.",
                )
                .long("single-file-name")
                .short("t"),
        )
        .arg(
            Arg::with_name("unsafe")
                .help("Unsafe mode. Use of this option is discouraged.")
                .long_help(
                    "Operate in \"unsafe\" mode.\n\nThe temp file will be unlinked before mktemp \
                     exits.\nThis is slightly better than mktemp(3) but still introduces a race \
                     condition.\n\nUse of this option is not encouraged.",
                )
                .long("unsafe")
                .visible_alias("dry-run")
                .short("u"),
        )
}
