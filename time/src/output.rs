//! Output interface for `time`

use coreutils_core::os::{resource::RUsage, TimeVal};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::default_formatter;

#[derive(Debug, PartialEq)]
pub enum OutputFormatter {
    Default,
    Posix,
}

/// Express `coreutils_core::os::TimeVal` into `f64` seconds
fn as_secs_f64(tv: TimeVal) -> f64 { tv.tv_sec as f64 + (tv.tv_usec as f64) / 1_000_000.0 }

impl OutputFormatter {
    pub fn format_stats(self, rusage: &RUsage, duration: &std::time::Duration) -> String {
        let wall_time = duration.as_secs_f64();
        let user_time = as_secs_f64(rusage.timing.user_time);
        let sys_time = as_secs_f64(rusage.timing.sys_time);
        match self {
            OutputFormatter::Default => default_formatter(rusage, wall_time, user_time, sys_time),
            OutputFormatter::Posix => {
                format!("real {:.2}\nuser {:.2}\nsys  {:.2}", wall_time, user_time, sys_time)
            },
        }
    }
}
