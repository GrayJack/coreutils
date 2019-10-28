use libc::{self, mode_t};
use std::{ffi::CString, io, io::Error, path::Path};

/// Mkfifo wrapper around libc's
/// Inspired by crate `unix_named_pipe`
pub fn mkfifo(filepath: &str, mode: u32) -> io::Result<()> {
    let path: &Path = Path::new(filepath);
    let path = CString::new(path.to_str().unwrap())?;
    let result = unsafe { libc::mkfifo(path.as_ptr(), mode as mode_t) };

    let result: i32 = result;
    if result == 0 {
        return Ok(());
    }
    Err(Error::last_os_error())
}
