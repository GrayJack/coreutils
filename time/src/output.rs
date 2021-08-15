//! Output interface for `time`

use std::fmt::Write;

use coreutils_core::os::{resource::RUsage, TimeVal};

#[derive(Debug, PartialEq)]
pub enum OutputFormatter {
    Default,
    Posix,
    CSH,
    TCSH,
    FmtString(String),
}

/// Express `coreutils_core::os::TimeVal` into `f64` seconds
fn as_secs_f64(tv: TimeVal) -> f64 {
    tv.tv_sec as f64 + (tv.tv_usec as f64) / 1_000_000.0
}

// Convenience struct for passing 3 floating point parameters
struct TimeTriple {
    pub user_time: f64,
    pub sys_time: f64,
    pub wall_time: f64,
}

impl OutputFormatter {
    pub fn format_stats(self, rusage: &RUsage, duration: &std::time::Duration) -> String {
        let timings: TimeTriple = TimeTriple {
            user_time: as_secs_f64(rusage.timing.user_time),
            sys_time: as_secs_f64(rusage.timing.sys_time),
            wall_time: duration.as_secs_f64(),
        };
        match self {
            OutputFormatter::Default => default_formatter(rusage, timings),
            OutputFormatter::Posix => {
                format!(
                    "real {:.2}\nuser {:.2}\nsys  {:.2}",
                    timings.wall_time, timings.user_time, timings.sys_time
                )
            },
            OutputFormatter::CSH => csh_formatter(rusage, timings),
            OutputFormatter::TCSH => tcsh_formatter(rusage, timings),
            OutputFormatter::FmtString(spec) => custom_formatter(rusage, timings, &spec),
        }
    }
}

fn default_formatter(_: &RUsage, timings: TimeTriple) -> String {
    format!(
        "{:.2} real {:.2} user {:.2} sys",
        timings.wall_time, timings.user_time, timings.sys_time
    )
}

/// Render the <specifier> in %<specifier>, return a pair of boolean and the rendered
/// The boolean signals if the specifier was rendered
fn render_percent_spec(rusage: &RUsage, timings: &TimeTriple, spec: char) -> Option<String> {
    match spec {
        'c' => Some(rusage.mem.num_invol_ctx_switch.to_string()),
        'D' => Some(rusage.mem.unshared_data_size.to_string()),
        'E' => Some(format!("{:.2}", timings.wall_time)),
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
            if timings.wall_time == 0_f64 {
                Some(String::from("0.0%"))
            } else {
                Some(format!("{:.2}", (timings.user_time + timings.sys_time) / timings.wall_time))
            }
        },
        'R' => Some(rusage.mem.num_minor_page_flt.to_string()),
        'r' => Some(rusage.io.num_sock_recv.to_string()),
        'S' => Some(format!("{:.2}", timings.sys_time)),
        's' => Some(rusage.io.num_sock_send.to_string()),
        'U' => Some(format!("{:.2}", timings.user_time)),
        'W' => Some(rusage.mem.num_swaps.to_string()),
        'w' => Some(rusage.mem.num_vol_ctx_switch.to_string()),
        'X' => {
            if timings.wall_time == 0_f64 {
                Some(String::from("0.0%"))
            } else {
                Some(rusage.mem.shared_mem_size.to_string())
            }
        },
        _ => None,
    }
}

fn custom_formatter(rusage: &RUsage, timings: TimeTriple, format_spec: &str) -> String {
    assert!(format_spec.is_ascii(), "Format string spec contains non-ascii characters");
    let mut target = String::new();
    // let characters: Vec<char> = format_spec.chars().collect();

    let mut format_spec_iterator = format_spec.chars().peekable();

    while let Some(ch) = format_spec_iterator.next() {
        if ch != '%' {
            write!(&mut target, "{}", ch).expect("Failed to write to format buffer");
        } else {
            match format_spec_iterator.peek() {
                Some(&specifier) => {
                    if let Some(text) = render_percent_spec(rusage, &timings, specifier) {
                        write!(&mut target, "{}", text).expect("Failed to write to format buffer");
                    } else {
                        write!(&mut target, "%{}", specifier)
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
        timings.user_time,
        timings.sys_time,
        timings.wall_time,
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
        timings.user_time,
        timings.sys_time,
        timings.wall_time,
        (timings.user_time + timings.sys_time) / timings.wall_time,
        rusage.mem.shared_mem_size,
        rusage.mem.unshared_stack_size,
        rusage.io.num_block_in,
        rusage.io.num_block_out,
        rusage.mem.num_major_page_flt,
        rusage.mem.num_swaps,
    )
}
