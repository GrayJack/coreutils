use clap::{
    crate_authors, crate_description, crate_name, crate_version, App,
    AppSettings::AllowNegativeNumbers, Arg,
};

pub(crate) fn create_app<'help>() -> App<'help> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .mut_arg("help", |help| help.help("Display help information.").short('?'))
        .mut_arg("version", |v| v.help("Display version information."))
        .setting(AllowNegativeNumbers)
        .arg(
            Arg::new("FILE")
                .help("Files to be used by the program.")
                .required(false)
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new("line-end-null")
                .help("Ends each output line with NUL, not newline.")
                .long("null")
                .short('0'),
        )
        .arg(
            Arg::new("all")
                .help("Display counts for all files, not just directories.")
                .long("all")
                .short('a'),
        )
        .arg(
            Arg::new("apparent-size")
                .help("Display apparent sizes rather than disk usage.")
                .long_help(
                    "Display apparent sizes rather than disk usage.\n\nThis can be helpful when \
                     operating on compressed volumes or sparse files.",
                )
                .long("apparent-size")
                .short('A'),
        )
        .arg(
            Arg::new("block-size")
                .help("Scales sizes by SIZE before displaying them.")
                .long_help(
                    "Scales sizes by SIZE before displaying them.\n\nCalculate block counts in \
                     BLOCKSIZE byte blocks.\n\nThis is different from the -h, -k, -m, --si and -g \
                     options or setting BLOCKSIZE and gives an estimate of how much space the \
                     examined file hierarchy would require on a filesystem with the given \
                     BLOCKSIZE.\n\nUnless in -A mode, BLOCKSIZE is rounded up to the next \
                     multiple of 512.",
                )
                .long("block-size")
                .short('B')
                .value_name("BLOCKSIZE"),
        )
        .arg(
            Arg::new("bytes")
                .help("Equivalent to '--apparent-size --block-size=1'.")
                .long("bytes")
                .short('b'),
        )
        .arg(Arg::new("total").help("Produces a grand total.").long("total").short('c'))
        .arg(
            Arg::new("dereference-args")
                .help("Dereference only symlinks that are given as arguments.")
                .long("dereference-args")
                .short('D'),
        )
        .arg(
            Arg::new("max-depth")
                .help(
                    "Display the total for a directory only if it is N or fewer levels below the \
                     FILEs",
                )
                .long("max-depth")
                .short('d')
                .value_name("N"),
        )
        .arg(
            Arg::new("dereference-args-alias")
                .help("Equivalent to '--dereference-args (-D)'")
                .long("deref-args")
                .short('H'),
        )
        .arg(
            Arg::new("human-readable")
                .help("Display sizes in a human readable format.")
                .long("human-readable")
                .short('h')
                .conflicts_with_all(&[
                    "si",
                    "apparent-size",
                    "block-size",
                    "block-size-k",
                    "block-size-m",
                    "bytes",
                ]),
        )
        .arg(
            Arg::new("inodes")
                .help("Lists inode usage information instead of block usage.")
                .long("inodes")
                .short('i')
                .conflicts_with_all(&[
                    "apparent-size",
                    "block-size",
                    "block-size-k",
                    "block-size-m",
                    "bytes",
                ]),
        )
        .arg(
            Arg::new("block-size-k")
                .help("Equivalent to '--block-size=1K'")
                .long("blocks-k")
                .short('k'),
        )
        .arg(
            Arg::new("block-size-m")
                .help("Equivalent to '--block-size=1M'.")
                .long("blocks-m")
                .short('m'),
        )
        .arg(
            Arg::new("dereference")
                .help("Dereferences all symbolic links.")
                .long_help(
                    "Dereferences all symbolic links.\n\nThis option overrides any previous -P.",
                )
                .long("dereference")
                .visible_alias("deref")
                .short('L')
                .overrides_with("no-dereference"),
        )
        .arg(
            Arg::new("no-dereference")
                .help("Do not follow any symbolic links (it's the default).")
                .long_help(
                    "Do not follow any symbolic links (it's the default).\n\nThis option \
                     overrides any previous -L.",
                )
                .long("no-dereference")
                .short('P')
                .overrides_with("dereference"),
        )
        .arg(
            Arg::new("count-links")
                .help("Counts sizes many times if hard linked.")
                .long_help(
                    "Counts sizes many times if hard linked.\n\nThe default behavior of du is to \
                     count files with multiple hard links only once.\n\nWhen the -l option is \
                     specified, the hard link checks are disabled, and these files are counted \
                     (and displayed) as many times as they are found.",
                )
                .long("count-links")
                .short('l'),
        )
        .arg(
            Arg::new("separate-dirs")
                .help("For directories do not include size of subdirectories.")
                .long("separate-dirs")
                .short('S'),
        )
        .arg(
            Arg::new("si")
                .help("Like -h, but uses powers of 1000 not 1024.")
                .long("si")
                .short('I')
                .conflicts_with_all(&[
                    "human-readable",
                    "apparent-size",
                    "block-size",
                    "block-size-k",
                    "block-size-m",
                    "bytes",
                ]),
        )
        .arg(
            Arg::new("summarize")
                .help("Display only a total for each FILE.")
                .long("summarize")
                .short('s')
                .conflicts_with("all"),
        )
        .arg(
            Arg::new("threshold")
                .help(
                    "Exclude entries smaller then SIZE if positive or entries greater than SIZE \
                     if negative.",
                )
                .long("threshold")
                .short('t')
                .value_name("SIZE"),
        )
        .arg(
            Arg::new("time")
                .help("Show WORD file timestamp.")
                .long("time")
                .short('T')
                .value_name("WORD")
                .possible_values(["mtime", "atime", "ctime", "access", "use", "status"]),
        )
        .arg(
            Arg::new("time-style")
                .help("Show times using STYLE.")
                .long("time-style")
                .short('j')
                .value_name("STYLE")
                .requires("time"),
        )
        .arg(
            Arg::new("exclude-pattern")
                .help("Exclude files that match PATTERN.")
                .long("exclude")
                .short('p')
                .value_name("PATTERN"),
        )
        .arg(
            Arg::new("one-file-system")
                .help("Skip directories on different file systems.")
                .long("one-file-system")
                .short('x'),
        )
}
