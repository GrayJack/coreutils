use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("logical")
                .help("Display the logical current working directory.")
                .long_help(
                    "Display the logical current working directory.\n\nUse PWD from environment, \
                     even if it contains symlinks.",
                )
                .long("logical")
                .short('l')
                .overrides_with("physical"),
        )
        .arg(
            Arg::new("physical")
                .help("Display the physical current working directory. Avoid all symlinks.")
                .long_help(
                    "Display the physical current working directory. Avoid all symlinks (All \
                     symbolic links resolved).",
                )
                .long("physical")
                .short('p')
                .overrides_with("logical"),
        )
}
