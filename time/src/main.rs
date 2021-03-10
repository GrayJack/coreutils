mod cli;
mod flags;
mod output;
mod subprocess;

use coreutils_core::os::resource::{get_rusage, ResourceConsumer};

fn main() {
    let opts = flags::TimeOpts::from_matches();
    let (exit_status, duration) = match subprocess::timed_run(&opts.command) {
        Ok(rv) => rv,
        Err(err) => subprocess::exit_with_msg(err),
    };

    let usage = get_rusage(ResourceConsumer::Children);

    eprintln!("{}", opts.printer.format_stats(&usage, &duration));
    std::process::exit(exit_status.code().unwrap_or(1));
}
