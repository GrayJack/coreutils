use libc::{self, mode_t, EACCES, EEXIST, ENOENT};
use std::{ffi::CString, io, io::Error, path::Path};

/// Mkfifo wrapper around libc's
/// Inspired by crate `unix_named_pipe`
pub fn mkfifo(filepath: &str, mode: u32) -> io::Result<()> {
    let path: &Path = Path::new(filepath).as_ref();
    let path = CString::new(path.to_str().unwrap())?;
    let result = unsafe { libc::mkfifo(path.as_ptr(), mode as mode_t) };

    let result: i32 = result;
    if result == 0 {
        return Ok(());
    }

    let error = errno::errno();
    match Error::last_os_error().raw_os_error().unwrap() {
        EACCES => {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("could not open {:?}: {}", path, error),
            ));
        },
        EEXIST => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("could not open {:?}: {}", path, error),
            ));
        },
        ENOENT => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("could not open {:?}: {}", path, error),
            ));
        },
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("could not open {:?}: {}", path, error),
            ));
        },
    }
}
