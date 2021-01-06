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
                .help("File(s) to write to.")
                .multiple(true),
        )
        .arg(
            Arg::with_name("append")
                .help("Append the output to the files.")
                .short("a")
                .long("append"),
        )
        .arg(Arg::with_name("ignore").help("Ignore interrupt signals.").short("i").long("ignore"))
}
