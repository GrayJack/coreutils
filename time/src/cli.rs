use clap::{
    crate_authors, crate_description, crate_name, crate_version, App,
    AppSettings::{ColoredHelp, TrailingVarArg},
    Arg,
};
use indoc::indoc;

const FMT_ARG_HELP: &str = indoc! {"
    Specify a time format using the csh(1) time builtin syntax. The
    following sequences are supported:
    %U    The time the process spent in user mode in cpu seconds.
    %S    The time the process spent in kernel mode in cpu seconds.
    %E    The elapsed (wall clock) time in seconds.
    %P    The CPU percentage computed as (%U + %S) / %E.
    %W    Number of times the process was swapped.
    %X    The average amount in (shared) text space used in Kbytes.
    %D    The average amount in (unshared) data/stack space used in
          Kbytes.
    %K    The total space used (%X + %D) in Kbytes.
    %M    The maximum memory the process had in use at any time in
          Kbytes.
    %F    The number of major page faults (page needed to be brought
          from disk).
    %R    The number of minor page faults.
    %I    The number of input operations.
    %O    The number of output operations.
    %r    The number of socket messages received.
    %s    The number of socket messages sent.
    %k    The number of signals received.
    %w    The number of voluntary context switches (waits).
    %c    The number of involuntary context switches.
"};

const CSH_FMT_HELP: &str = indoc! {"
    Displays information in the format used by default the time
    builtin of csh(1) uses (%Uu  %Ss %E %P %X+%Dk %I+%Oio %Fpf+%Ww)
"};

const POSIX_FMT_HELP: &str = indoc! {"
    Display time output in POSIX specified format as:
        real %f
        user %f
        sys %f
    Timer accuracy is arbitrary, but will always be counted in seconds.
"};

const TCSH_FMT_HELP: &str = indoc! {"
    Displays information in the format used by default the time
    builtin of tcsh(1) uses (%Uu %Ss %E %P\\t%X+%Dk %I+%Oio %Fpf+%Ww)
    with three decimal places for time values.
"};

pub(crate) fn create_app<'a, 'b>() -> App<'a, 'b> {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .help_message("Display help information.")
        .version_message("Display version information.")
        .help_short("?")
        .settings(&[ColoredHelp, TrailingVarArg])
        .arg(
            Arg::with_name("COMMAND")
                .help("Command to run and it's arguments.")
                .multiple(true)
                .required(true),
        )
        .arg(Arg::with_name("posix").help(POSIX_FMT_HELP).long("posix").short("p"));
    configure_extensions(app)
}

#[rustfmt::skip]
fn configure_extensions<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    let use_csh_fmt = Arg::with_name("use_csh_fmt")
        .conflicts_with_all(&["posix"])
        .help(CSH_FMT_HELP)
        .long("csh-format")
        .short("c");
    let format_string = Arg::with_name("format_string")
        .conflicts_with_all(&["posix", "use_csh_fmt"])
        .takes_value(true)
        .validator(|s: String| -> Result<(), String> {
            if s.is_empty() || !s.is_ascii() {
                Err(String::from("Format string must be non-empty ASCII"))
            } else {
                Ok(())
            }
        })
        .help(FMT_ARG_HELP)
        .long("format")
        .short("f");
    let use_tcsh_fmt = Arg::with_name("use_tcsh_fmt")
        .conflicts_with_all(&["posix", "use_csh_fmt", "format_string"])
        .help(TCSH_FMT_HELP)
        .long("tcsh-format")
        .short("t");

    let dump_rusage = Arg::with_name("dump_rusage")
        .conflicts_with_all(&["posix"])
        .help("Lists resource utilization information. The contents of \
              the\ncommand process's rusage structure are printed")
        .long("rusage")
        .short("l");

    let human_readable = Arg::with_name("human_readable")
        .conflicts_with("posix")
        .help("Time durations are printed in hours, minutes, seconds")
        .long("human-readable")
        .short("h");

    let output_path = Arg::with_name("output_file")
        .takes_value(true)
        .help("Write the output to file instead of stderr.\
              If file exists and the -a flag is not specified,\
              the file will be overwritten.")
        .long("output-path")
        .short("o");

    let append_output = Arg::with_name("append_mode")
        .help("If the -o flag is used, append to specified file rather that\
              overwrite it. Otherwise this option has no effect")
        .long("append-mode")
        .short("a");

    if cfg!(target_os = "netbsd") {
        app.args(&[use_csh_fmt, format_string, dump_rusage, use_tcsh_fmt])
    } else if cfg!(any(target_os = "freebsd", target_os = "dragonflybsd", target_os = "macos")) {
        app.args(&[append_output, dump_rusage, human_readable, output_path])
    } else if cfg!(any(target_os = "openbsd", target_os = "macos")) {
        app.arg(dump_rusage)
    } else {
        app
    }
}
