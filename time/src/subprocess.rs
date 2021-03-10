/// Module for creating, and interacting with child processes
use std::process::{exit, Command, ExitStatus, Stdio};
use std::{
    io,
    time::{Duration, Instant},
};

type SubprocessTiming = (ExitStatus, Duration);

/// Wrapper around `std::process::exit` that prints the error's
/// message to stderr before quitting.
///
/// Will try to propagate the error code set in the err if available
pub fn exit_with_msg(err: std::io::Error) -> ! {
    eprintln!("{}", err);

    // Translate the exit code according to POSIX spec
    // 1-125: for errors internal to `time`
    // 126  : Command was found but could not be invoked (PermissionError)
    // 127  : Command was not found
    exit(match err.kind() {
        io::ErrorKind::PermissionDenied => 126,
        io::ErrorKind::NotFound => 127,
        // Translate other error code to 0-124 and shift right by 1
        // Internal exit codes are typically arbitrary enough that they be
        // considered limited to developer use-only
        _ => 1 + (err.raw_os_error().unwrap_or(0) % 125),
    })
}

/// Wrapper for creating, spawning and waiting on `std::process::Command`
/// Returns the `std::process::ExitStatus` of the `std::process::Command`
/// that was run
pub fn timed_run(cmd_slice: &[String]) -> io::Result<SubprocessTiming> {
    let mut cmd = Command::new(&cmd_slice[0]);
    cmd.args(&cmd_slice[1..]);
    cmd.stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit());

    let start_time = Instant::now();
    let status = cmd.spawn()?.wait()?;
    Ok((status, start_time.elapsed()))
}

#[cfg(test)]
mod tests {
    use super::timed_run;

    #[test]
    fn invalid_command_returns_errno_when_set() {
        if let Err(err) = timed_run(&["does-not-exist".to_string()]) {
            assert!(err.raw_os_error() == Some(2))
        } else {
            panic!("Subprocess did not fail as expected")
        }
    }
}
