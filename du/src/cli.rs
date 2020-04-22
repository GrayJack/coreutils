use clap::{
    crate_authors, crate_description, crate_name, crate_version, App,
    AppSettings::{AllowNegativeNumbers, ColoredHelp},
    Arg,
};

pub(crate) fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .help_message("Display help information.")
        .version_message("Display version information.")
        .help_short("?")
        .settings(&[ColoredHelp, AllowNegativeNumbers])
        .arg(
            Arg::with_name("FILE")
                .help("Files to be used by the program.")
                .required(false)
                .multiple(true),
        )
        .arg(
            Arg::with_name("line-end-null")
                .help("Ends each output line with NUL, not newline.")
                .long("null")
                .short("0"),
        )
        .arg(
            Arg::with_name("all")
                .help("Display counts for all files, not just directories.")
                .long("all")
                .short("a"),
        )
        .arg(
            Arg::with_name("apparent-size")
                .help("Display apparent sizes rather than disk usage.")
                .long_help(
                    "Display apparent sizes rather than disk usage.\n\nThis can be helpful when \
                     operating on compressed volumes or sparse files.",
                )
                .long("apparent-size")
                .short("A"),
        )
        .arg(
            Arg::with_name("block-size")
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
                .short("B")
                .value_name("BLOCKSIZE"),
        )
        .arg(
            Arg::with_name("bytes")
                .help("Equivalent to '--apparent-size --block-size=1'.")
                .long("bytes")
                .short("b"),
        )
        .arg(Arg::with_name("total").help("Produces a grand total.").long("total").short("c"))
        .arg(
            Arg::with_name("dereference-args")
                .help("Dereference only symlinks that are given as arguments.")
                .long("dereference-args")
                .short("D"),
        )
        .arg(
            Arg::with_name("max-depth")
                .help(
                    "Display the total for a directory only if it is N or fewer levels below the \
                     FILEs",
                )
                .long("max-depth")
                .short("d")
                .value_name("N"),
        )
        .arg(
            Arg::with_name("dereference-args-alias")
                .help("Equivalent to '--dereference-args (-D)'")
                .long("deref-args")
                .short("H"),
        )
        .arg(
            Arg::with_name("human-readable")
                .help("Display sizes in a human readable format.")
                .long("human-readable")
                .short("h")
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
            Arg::with_name("inodes")
                .help("Lists inode usage information instead of block usage.")
                .long("inodes")
                .short("i")
                .conflicts_with_all(&[
                    "apparent-size",
                    "block-size",
                    "block-size-k",
                    "block-size-m",
                    "bytes",
                ]),
        )
        .arg(
            Arg::with_name("block-size-k")
                .help("Equivalent to '--block-size=1K'")
                .long("blocks-k")
                .short("k"),
        )
        .arg(
            Arg::with_name("block-size-m")
                .help("Equivalent to '--block-size=1M'.")
                .long("blocks-m")
                .short("m"),
        )
        .arg(
            Arg::with_name("dereference")
                .help("Dereferences all symbolic links.")
                .long_help(
                    "Dereferences all symbolic links.\n\nThis option overrides any previous -P.",
                )
                .long("dereference")
                .visible_alias("deref")
                .short("L")
                .overrides_with("no-dereference"),
        )
        .arg(
            Arg::with_name("no-dereference")
                .help("Do not follow any symbolic links (it's the default).")
                .long_help(
                    "Do not follow any symbolic links (it's the default).\n\nThis option \
                     overrides any previous -L.",
                )
                .long("no-dereference")
                .short("P")
                .overrides_with("dereference"),
        )
        .arg(
            Arg::with_name("count-links")
                .help("Counts sizes many times if hard linked.")
                .long_help(
                    "Counts sizes many times if hard linked.\n\nThe default behavior of du is to \
                     count files with multiple hard links only once.\n\nWhen the -l option is \
                     specified, the hard link checks are disabled, and these files are counted \
                     (and displayed) as many times as they are found.",
                )
                .long("count-links")
                .short("l"),
        )
        .arg(
            Arg::with_name("separate-dirs")
                .help("For directories do not include size of subdirectories.")
                .long("separate-dirs")
                .short("S"),
        )
        .arg(
            Arg::with_name("si")
                .help("Like -h, but uses powers of 1000 not 1024.")
                .long("si")
                .short("I")
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
            Arg::with_name("summarize")
                .help("Display only a total for each FILE.")
                .long("summarize")
                .short("s")
                .conflicts_with("all"),
        )
        .arg(
            Arg::with_name("threshold")
                .help(
                    "Exclude entries smaller then SIZE if positive or entries greater than SIZE \
                     if negative.",
                )
                .long("threshold")
                .short("t")
                .value_name("SIZE"),
        )
        .arg(
            Arg::with_name("time")
                .help("Show WORD file timestamp.")
                .long("time")
                .short("T")
                .value_name("WORD")
                .possible_values(&["mtime", "atime", "ctime", "access", "use", "status"]),
        )
        .arg(
            Arg::with_name("time-style")
                .help("Show times using STYLE.")
                .long("time-style")
                .short("j")
                .value_name("STYLE")
                .requires("time"),
        )
        .arg(
            Arg::with_name("exclude-pattern")
                .help("Exclude files that match PATTERN.")
                .long("exclude")
                .short("p")
                .value_name("PATTERN"),
        )
        .arg(
            Arg::with_name("one-file-system")
                .help("Skip directories on different file systems.")
                .long("one-file-system")
                .short("x"),
        )
}
