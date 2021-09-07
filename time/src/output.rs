//! The Output interface for `time` is detailed in this module

use coreutils_core::time::Duration;
use std::fmt::Write;
use coreutils_core::os::resource::RUsage;

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
    CSH,

    /// Display time output in the tcsh(1) default format:
    /// ```
    /// %Uu %Ss %E %P\t%X+%Dk %I+%Oio %Fpf+%Ww
    /// ```
    TCSH,

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
    pub fn format_stats(self, rusage: &RUsage, duration: &Duration) -> String {
        let timings: TimeTriple = TimeTriple {
            user_time: rusage.timing.user_time,
            sys_time: rusage.timing.sys_time,
            wall_time: *duration,
        };
        match self.kind {
            FormatterKind::Default => default_formatter(rusage, timings),
            FormatterKind::Posix => {
                format!(
                    "real {:.2}\nuser {:.2}\nsys  {:.2}",
                    timings.wall_time.as_seconds_f64(),
                    timings.user_time.as_seconds_f64(),
                    timings.sys_time.as_seconds_f64()
                )
            },
            FormatterKind::CSH => csh_formatter(rusage, timings),
            FormatterKind::TCSH => tcsh_formatter(rusage, timings),
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
fn render_percent_spec(rusage: &RUsage, timings: &TimeTriple, spec: u8) -> Option<String> {
    match spec {
        b'c' => Some(rusage.mem.num_invol_ctx_switch.to_string()),
        b'D' => Some(rusage.mem.unshared_data_size.to_string()),
        b'E' => Some(format!("{:.2}", timings.wall_time.as_seconds_f64())),
        b'F' => Some(rusage.mem.num_major_page_flt.to_string()),
        b'I' => Some(rusage.io.num_block_in.to_string()),
        b'K' => Some(
            (rusage.mem.shared_mem_size
                + rusage.mem.unshared_data_size
                + rusage.mem.unshared_stack_size)
                .to_string(),
        ),
        b'k' => Some(rusage.io.num_signals.to_string()),
        b'M' => Some(rusage.mem.max_rss.to_string()),
        b'O' => Some(rusage.mem.max_rss.to_string()),
        b'P' => {
            if timings.wall_time.is_zero() {
                Some(String::from("0.0%"))
            } else {
                let cpu_time: Duration = timings.user_time + timings.sys_time;
                Some(format!("{:.2}", 100 * cpu_time / timings.wall_time))
            }
        },
        b'R' => Some(rusage.mem.num_minor_page_flt.to_string()),
        b'r' => Some(rusage.io.num_sock_recv.to_string()),
        b'S' => Some(format!("{:.2}", timings.sys_time.as_seconds_f64())),
        b's' => Some(rusage.io.num_sock_send.to_string()),
        b'U' => Some(format!("{:.2}", timings.user_time.as_seconds_f64())),
        b'W' => Some(rusage.mem.num_swaps.to_string()),
        b'w' => Some(rusage.mem.num_vol_ctx_switch.to_string()),
        b'X' => Some(rusage.mem.shared_mem_size.to_string()),
        _ => None,
    }
}

/// Internal: Unescapes a backslash escaped string
/// See: https://en.wikipedia.org/wiki/Escape_sequences_in_C#Table_of_escape_sequences
fn decode_escaped_string(value: String) -> String {
    let mut iter = value.bytes().peekable();
    let mut output = String::new();

    while let Some(ch) = iter.next() {
        if ch == b'\\' {
            if let Some(next_ch) = iter.peek() {
                // Find out which of the escape codes was used to decide
                // which byte to write next. If none of the escape codes match,
                // write the backslash that was going to be skipped instead
                let (decode_successful, byte_to_write): (bool, u8) = match next_ch {
                    b'a' => (true, 0x07),
                    b'b' => (true, 0x08),
                    b'e' => (true, 0x1B),
                    b'f' => (true, 0x0C),
                    b'n' => (true, 0x0A),
                    b'r' => (true, 0x0D),
                    b't' => (true, 0x09),
                    b'v' => (true, 0x0B),
                    b'\\' => (true, 0x5C),
                    b'\'' => (true, 0x27),
                    b'\"' => (true, 0x22),
                    b'?' => (true, 0x3F),
                    _ => (false, b'\\'),
                };

                write!(output, "{}", byte_to_write as char).expect("Failed to unescape string");
                // If a byte was decoded, skip over it.
                // Otherwise, write out the backslash
                if decode_successful {
                    iter.next();
                }
            } else {
                panic!("Invalid escape '\\' at end of input");
            }
        } else {
            write!(output, "{}", ch as char).expect("Failed to unescape string");
        }
    }
    output
}

fn custom_formatter(rusage: &RUsage, timings: TimeTriple, format_spec: &str) -> String {
    let mut target = String::new();
    let unescaped_spec = decode_escaped_string(format_spec.to_owned());
    let mut format_spec_iterator = unescaped_spec.bytes().peekable();

    while let Some(ch) = format_spec_iterator.next() {
        if ch != b'%' {
            write!(&mut target, "{}", ch as char).expect("Failed to write to format buffer");
        } else {
            match format_spec_iterator.peek() {
                Some(&specifier) => {
                    if let Some(text) = render_percent_spec(rusage, &timings, specifier) {
                        write!(&mut target, "{}", text).expect("Failed to write to format buffer");
                    } else {
                        // If the %<char> wasn't rendered, dump it out as it was seen
                        write!(&mut target, "%{}", specifier as char)
                            .expect("Failed to write to format buffer");
                    }
                    // Skip this character, we have dealt with the result of .peek()
                    format_spec_iterator.next();
                },
                None => {
                    write!(&mut target, "%").expect("Failed to write to format buffer");
                },
            }
        }
    }
    target
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
