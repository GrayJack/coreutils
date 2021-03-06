//! Output interface for `time`

#[derive(Debug, PartialEq)]
pub enum OutputFormatter {
    Default,
    Posix,
}

impl OutputFormatter {
    pub fn format_stats(rusage: &Rusage, duration: &std::time::Duration) {}
}
