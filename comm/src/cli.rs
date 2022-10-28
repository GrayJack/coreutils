use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .arg(
            Arg::new("1")
                .short('1')
                .help("Suppress lines unique to file1.")
                
        )
        .arg(
            Arg::new("2")
                .short('2')
                .help("Suppress lines unique to file2.")
        )
        .arg(
            Arg::new("3")
                .short('3')
                .help("Suppress lines common to both files.")
        )
        .arg(
            Arg::new("file_1")
                .required(true)
                .value_hint(clap::ValueHint::FilePath),
        )
        .arg(
            Arg::new("file_2")
                .required(true)
                .value_hint(clap::ValueHint::FilePath),
        )
}
