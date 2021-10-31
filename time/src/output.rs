//! The Output interface for `time` is detailed in this module

use std::fmt::Write;

use coreutils_core::{os::resource::RUsage, time::Duration};

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
            FormatterKind::Csh => csh_formatter(rusage, timings),
            FormatterKind::TCsh => tcsh_formatter(rusage, timings),
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

/// Wraps a character iterator over a string,
/// and unescapes escaped ASCII control sequences as it comes across them.
///
/// A simple example of this would be rendering escaped newline and tab chars:
///
/// ```rust
/// let s = StringEscapeDecoder::from(&"a\\nb\\tc")
/// assert_eq!("a\nb\tc", a.collect());
/// ```
///
/// See: https://en.wikipedia.org/wiki/Escape_sequences_in_C#Table_of_escape_sequences
struct StringEscapeDecoder<'a> {
    data: std::iter::Peekable<std::str::Chars<'a>>,
    min_size: usize,
    max_size: usize,
}

impl<'a> From<&'a str> for StringEscapeDecoder<'a> {
    fn from(buffer: &'a str) -> Self {
        let max_escapes = buffer.chars().filter(|&c| c == '\\').count();
        StringEscapeDecoder {
            data: buffer.chars().peekable(),
            min_size: buffer.len() - max_escapes,
            max_size: buffer.len(),
        }
    }
}

impl<'a> std::iter::Iterator for StringEscapeDecoder<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.data.next() {
            Some('\\') => {
                let (consume_next, char_to_emit) = match self.data.peek() {
                    Some('0') => (true, /* '\0' */ 0x00 as char),
                    Some('a') => (true, /* '\a' */ 0x07 as char),
                    Some('b') => (true, /* '\b' */ 0x08 as char),
                    Some('e') => (true, /* '\e' */ 0x1B as char),
                    Some('f') => (true, /* '\f' */ 0x0C as char),
                    Some('n') => (true, '\n'),
                    Some('r') => (true, '\r'),
                    Some('t') => (true, '\t'),
                    Some('v') => (true, /* '\v' */ 0x0B as char),
                    Some('\\') => (true, '\\'),
                    Some('\'') => (true, '\''),
                    Some('"') => (true, '"'),
                    Some('?') => (true, /* '\?' */ 0x3F as char),
                    // If the next character isn't a known ASCII escape code,
                    // or doesn't exist: Return the current '\\'
                    _ => (false, '\\'),
                };

                if consume_next {
                    self.data.next();
                    self.min_size -= 2;
                    self.max_size -= 2;
                } else {
                    self.min_size -= 1;
                    self.max_size -= 1;
                };
                Some(char_to_emit)
            },
            Some(c) => Some(c),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.min_size, Some(self.max_size))
    }
}

fn custom_formatter(rusage: &RUsage, timings: TimeTriple, format_spec: &str) -> String {
    let mut target = String::new();
    let mut format_spec_iterator = StringEscapeDecoder::from(format_spec).peekable();

    while let Some(ch) = format_spec_iterator.next() {
        if ch != '%' {
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
