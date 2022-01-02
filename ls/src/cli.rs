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
                .help("File(s) to list")
                .required(true)
                .multiple_occurrences(true)
                .default_value("."),
        )
        .arg(
            Arg::new("all")
                .help(
                    "Write out all directory entries, including those whose names begin with a \
                     <period> ( '.' ).",
                )
                .short('a')
                .long("all"),
        )
        .arg(
            Arg::new("almost_all")
                .help(
                    "Write out all directory entries, including those whose names begin with a \
                     <period> ( '.' ) but excluding the entries dot and dot-dot (if they exist).",
                )
                .short('A')
                .long("almost-all"),
        )
        .arg(
            Arg::new("file_status_modification")
                .help(
                    "Use time of last modification of the file status information instead of last \
                     modification of the file itself for sorting -t or writing -l.",
                )
                .short('c')
                .long("file-status-modification"),
        )
        .arg(
            Arg::new("order_top_to_bottom")
                .help("Write multi-text-column output with entries sorted down the columns.")
                .short('C')
                .long("order-top-to-bottom"),
        )
        .arg(
            Arg::new("directory")
                .help("List directories and files themselves, rather than their contents.")
                .short('d')
                .long("directory"),
        )
        .arg(
            Arg::new("no_sort")
                .help(
                    "Output is not sorted. This option turns on -a. It also negates the effect of \
                     the -r, -S and -t options.",
                )
                .short('f')
                .long("no-sort"),
        )
        .arg(
            Arg::new("classify")
                .help("Append indicator (one of */=>@|) to entries.")
                .short('F')
                .long("classify"),
        )
        .arg(
            Arg::new("no_dereference")
                .help("Follow symbolic links listed on the command line.")
                .short('H')
                .long("no-dereference"),
        )
        .arg(
            Arg::new("block_size")
                .help(
                    "Set the block size for the -s option and the per-directory block count \
                     written for the -l, -n, -s, -g, and -o options to 1024 bytes.",
                )
                .short('k')
                .long("block-size"),
        )
        .arg(
            Arg::new("comma_separate")
                .help("Fill width with a comma separated list of entries.")
                .short('m')
                .long("comma-separate"),
        )
        .arg(
            Arg::new("dereference")
                .help(
                    "When showing file information for a symbolic link, show information for the \
                     file the link references rather than for the link itself.",
                )
                .short('L')
                .long("dereference"),
        )
        .arg(
            Arg::new("indicator")
                .help("Write a <slash> ( '/' ) after each filename if that file is a directory.")
                .short('p')
                .long("indicator"),
        )
        .arg(
            Arg::new("inode")
                .help("For each file, write the file's file serial number.")
                .short('i')
                .long("inode"),
        )
        .arg(
            Arg::new("last_accessed")
                .help(
                    "Use time of last access instead of last modification of the file for sorting \
                     -t or writing -l.",
                )
                .short('u')
                .long("last-accessed"),
        )
        .arg(Arg::new("list").help("Use a long listing format").short('l').long("list"))
        .arg(
            Arg::new("no_owner")
                .help("Like -l, but do not list owner.")
                .short('g')
                .long("no-owner"),
        )
        .arg(
            Arg::new("numeric_uid_gid")
                .help("Like -l, but list numeric user and group IDs.")
                .short('n')
                .long("numeric-uid-gid"),
        )
        .arg(
            Arg::new("no_group")
                .help("Like -l, but do not list group.")
                .short('o')
                .long("no-group"),
        )
        .arg(
            Arg::new("hide_control_chars")
                .help(
                    "Force each instance of non-printable filename characters to be written as \
                     the � character. This is the default behavior if the output is to a terminal \
                     device.",
                )
                .short('q')
                .long("hide-control-chars"),
        )
        .arg(Arg::new("reverse").help("Reverse order while sorting.").short('r').long("reverse"))
        .arg(
            Arg::new("recursive")
                .help("Recursively print subdirectories.")
                .short('R')
                .long("recursive"),
        )
        .arg(
            Arg::new("size")
                .help("Print the allocated size of each file, in blocks.")
                .short('s')
                .long("size"),
        )
        .arg(
            Arg::new("sort_size")
                .help("Sort by first file size, largest first.")
                .short('S')
                .long("sort-size"),
        )
        .arg(
            Arg::new("time")
                .help("Sort by modification time, newest first.")
                .short('t')
                .long("time"),
        )
        .arg(
            Arg::new("order_left_to_right")
                .help("Sort columns left to right.")
                .short('x')
                .long("order-left-to-right"),
        )
        .arg(
            Arg::new("one_per_line")
                .help("Force output to be one entry per line.")
                .short('1')
                .long("one-per-line"),
        )
}
