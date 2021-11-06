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

    let written = match opts.get_output_stream() {
        Ok(mut stream) => {
            let data = opts.printer.format_stats(&usage, &duration);
            stream.write(data.unwrap().as_bytes())
        },
        Err(err) => Err(err),
    };

    if let Err(err) = written {
        subprocess::exit_with_msg(err);
    }
    std::process::exit(exit_status.code().unwrap_or(1));
}
