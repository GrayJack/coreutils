//! Helper functions to deal with FIFO special file.

use std::{ffi::CString, io, io::Error, path::Path};

use libc::{self, mode_t};

/// Creates FIFO special file with name `filepath` with `mode` permissions.
/// Inspired by crate `unix_named_pipe`
///
/// # Errors
/// If a internal call set a errno (I/O OS error), an error variant will be returned.
#[inline]
pub fn mkfifo(filepath: &str, mode: u32) -> io::Result<()> {
    let path: &Path = Path::new(filepath);
    let path = CString::new(path.to_str().unwrap())?;
    let result = unsafe { libc::mkfifo(path.as_ptr(), mode as mode_t) };

    match result {
        0 => Ok(()),
        _ => Err(Error::last_os_error()),
    }
}
