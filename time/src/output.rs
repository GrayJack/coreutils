//! The Output interface for `time` is detailed in this module

use std::fmt::Write;

use coreutils_core::{os::resource::RUsage, strings::StringEscapeDecoder, time::Duration};

/// The `FormatterKind` enum is how `time` controls the printing
/// of timing and resource usage information
/// Timer accuracy is arbitrary, but will always be counted in seconds.
#[derive(Debug, PartialEq)]
pub enum FormatterKind {
    /// Display time output in the default format:
    /// `    %E real %U user %S sys`
    Default,

    /// Display time output in POSIX specified format as:
    /// ```
    ///     real %E
    ///     user %U
    ///     sys %S
    /// ```
    Posix,

    /// Display time output in the csh(1) default format:
    /// ```
    /// %Uu %Ss %E %P %X+%Dk %I+%Oio %Fpf+%Ww
    /// ```
    Csh,

    /// Display time output in the tcsh(1) default format:
    /// ```
    /// %Uu %Ss %E %P\t%X+%Dk %I+%Oio %Fpf+%Ww
    /// ```
    TCsh,

    /// Use a custom format string to render the output using printf-style
    /// `%<specifier>` markers. Supported markers are:
    /// ```
    ///     %U    The time the process spent in user mode in cpu seconds.
    ///     %S    The time the process spent in kernel mode in cpu seconds.
    ///     %E    The elapsed (wall clock) time in seconds.
    ///     %P    The CPU percentage computed as (%U + %S) / %E.
    ///     %W    Number of times the process was swapped.
    ///     %X    The average amount in (shared) text space used in Kbytes.
    ///     %D    The average amount in (unshared) data/stack space used in
    ///           Kbytes.
    ///     %K    The total space used (%X + %D) in Kbytes.
    ///     %M    The maximum memory the process had in use at any time in
    ///           Kbytes.
    ///     %F    The number of major page faults (page needed to be brought
    ///           from disk).
    ///     %R    The number of minor page faults.
    ///     %I    The number of input operations.
    ///     %O    The number of output operations.
    ///     %r    The number of socket messages received.
    ///     %s    The number of socket messages sent.
    ///     %k    The number of signals received.
    ///     %w    The number of voluntary context switches (waits).
    ///     %c    The number of involuntary context switches.
    /// ```
    FmtString(String),
}


/// The `OutputFormatter` collects all information required by `time`
/// to format the child process's measurments
#[derive(Debug)]
pub struct OutputFormatter {
    pub kind: FormatterKind,
    pub human_readable: bool,
}

/// Convenience struct for passing 3 duration parameters
struct TimeTriple {
    pub user_time: Duration,
    pub sys_time: Duration,
    pub wall_time: Duration,
}

impl OutputFormatter {
    /// Format the rusage and timing information into a `String`
    ///
    /// # Arguments
    ///
    /// * `rusage` - Resource usage of the process being timed
    /// * `duration` - Time taken by the process being timed
    pub fn format_stats(self, rusage: &RUsage, duration: &Duration) -> Option<String> {
        let timings: TimeTriple = TimeTriple {
            user_time: rusage.timing.user_time,
            sys_time: rusage.timing.sys_time,
            wall_time: *duration,
        };
        match self.kind {
            FormatterKind::Default => Some(default_formatter(rusage, timings)),
            FormatterKind::Posix => Some(format!(
                "real {:.2}\nuser {:.2}\nsys  {:.2}",
                timings.wall_time.as_seconds_f64(),
                timings.user_time.as_seconds_f64(),
                timings.sys_time.as_seconds_f64()
            )),
            FormatterKind::Csh => Some(csh_formatter(rusage, timings)),
            FormatterKind::TCsh => Some(tcsh_formatter(rusage, timings)),
            FormatterKind::FmtString(spec) => custom_formatter(rusage, timings, &spec),
        }
    }
}

fn default_formatter(_: &RUsage, timings: TimeTriple) -> String {
    format!(
        "{:.2} real {:.2} user {:.2} sys",
        timings.wall_time.as_seconds_f64(),
        timings.user_time.as_seconds_f64(),
        timings.sys_time.as_seconds_f64()
    )
}

/// Render the <specifier> in %<specifier>, return a pair of boolean and the rendered
/// The boolean signals if the specifier was rendered
fn render_percent_spec(rusage: &RUsage, timings: &TimeTriple, spec: char) -> Option<String> {
    match spec {
        'c' => Some(rusage.mem.num_invol_ctx_switch.to_string()),
        'D' => Some(rusage.mem.unshared_data_size.to_string()),
        'E' => Some(format!("{:.2}", timings.wall_time.as_seconds_f64())),
        'F' => Some(rusage.mem.num_major_page_flt.to_string()),
        'I' => Some(rusage.io.num_block_in.to_string()),
        'K' => Some(
            (rusage.mem.shared_mem_size
                + rusage.mem.unshared_data_size
                + rusage.mem.unshared_stack_size)
                .to_string(),
        ),
        'k' => Some(rusage.io.num_signals.to_string()),
        'M' => Some(rusage.mem.max_rss.to_string()),
        'O' => Some(rusage.mem.max_rss.to_string()),
        'P' => {
            if timings.wall_time.is_zero() {
                Some(String::from("0.0%"))
            } else {
                let cpu_time: Duration = timings.user_time + timings.sys_time;
                Some(format!("{:.2}", 100 * cpu_time / timings.wall_time))
            }
        },
        'R' => Some(rusage.mem.num_minor_page_flt.to_string()),
        'r' => Some(rusage.io.num_sock_recv.to_string()),
        'S' => Some(format!("{:.2}", timings.sys_time.as_seconds_f64())),
        's' => Some(rusage.io.num_sock_send.to_string()),
        'U' => Some(format!("{:.2}", timings.user_time.as_seconds_f64())),
        'W' => Some(rusage.mem.num_swaps.to_string()),
        'w' => Some(rusage.mem.num_vol_ctx_switch.to_string()),
        'X' => Some(rusage.mem.shared_mem_size.to_string()),
        _ => None,
    }
}

fn custom_formatter(rusage: &RUsage, timings: TimeTriple, format_spec: &str) -> Option<String> {
    let mut target = String::new();
    let mut format_spec_iterator = StringEscapeDecoder::from(format_spec).peekable();

    while let Some(ch) = format_spec_iterator.next() {
        if ch != '%' {
            write!(&mut target, "{}", ch as char).ok()?;
        } else {
            match format_spec_iterator.peek() {
                Some(&specifier) => {
                    if let Some(text) = render_percent_spec(rusage, &timings, specifier) {
                        write!(&mut target, "{}", text).ok()?;
                    } else {
                        // If the %<char> wasn't rendered, dump it out as it was seen
                        write!(&mut target, "%{}", specifier as char).ok()?;
                    }
                    // Skip this character, we have dealt with the result of .peek()
                    format_spec_iterator.next();
                },
                None => {
                    write!(&mut target, "%").ok()?;
                },
            }
        }
    }
    Some(target)
}

fn csh_formatter(rusage: &RUsage, timings: TimeTriple) -> String {
    format!(
        "{:.2}u {:.2}s {:.2} {:.2} {}+{}k {}+{}io {}pf+{}w",
        timings.user_time.as_seconds_f64(),
        timings.sys_time.as_seconds_f64(),
        timings.wall_time.as_seconds_f64(),
        (timings.user_time + timings.sys_time) / timings.wall_time,
        rusage.mem.shared_mem_size,
        rusage.mem.unshared_stack_size,
        rusage.io.num_block_in,
        rusage.io.num_block_out,
        rusage.mem.num_major_page_flt,
        rusage.mem.num_swaps,
    )
}

fn tcsh_formatter(rusage: &RUsage, timings: TimeTriple) -> String {
    format!(
        "{:.2}u {:.2}s {:.2} {:.2}\t{}+{}k {}+{}io {}pf+{}w",
        timings.user_time.as_seconds_f64(),
        timings.sys_time.as_seconds_f64(),
        timings.wall_time.as_seconds_f64(),
        100 * (timings.user_time + timings.sys_time) / timings.wall_time,
        rusage.mem.shared_mem_size,
        rusage.mem.unshared_stack_size,
        rusage.io.num_block_in,
        rusage.io.num_block_out,
        rusage.mem.num_major_page_flt,
        rusage.mem.num_swaps,
    )
}
