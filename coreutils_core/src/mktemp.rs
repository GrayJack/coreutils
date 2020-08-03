//! Module with wrappers for libc mkstemp(3), mkdtemp(3).

use std::{
    fmt::{self, Display},
    fs::File,
    io::{self, Error as IOError},
    os::unix::io::FromRawFd,
};

/// A struct that represents a mktemp(3) result.
/// This includes the file created, and the path to that file.
#[derive(Debug)]
pub struct Mktemp {
    pub file: File,
    pub path: String,
}

impl Display for Mktemp {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.pad(&self.path) }
}

/// Creates a temporary file based on the given `template`.
///
/// The `template` should end with a number of `X` characters.
///
/// Some libc implementations requires the `template` to end with a minimum number of X
/// characters (for example, 6 in glibc as of version 2.29).
///
/// # Errors
/// If a internal call set a errno (I/O OS error), an error variant will be returned.
#[inline]
pub fn mkstemp(template: &str) -> io::Result<Mktemp> {
    let mut template_cstr = {
        let mut t = String::new();
        t.push_str(template);
        t.push('\0');
        t
    };

    let fd = unsafe { libc::mkstemp(template_cstr.as_mut_ptr() as *mut libc::c_char) };

    match fd {
        -1 => Err(IOError::last_os_error()),
        _ => {
            template_cstr.pop();
            Ok(Mktemp { file: unsafe { File::from_raw_fd(fd) }, path: template_cstr })
        },
    }
}

/// Creates a temporary directory based on the given `template`.
///
/// The `template` should end with a number of `X` characters.
///
/// Some libc implementations requires the `template` to end with a minimum number of X
/// characters (for example, 6 in glibc as of version 2.29).
///
/// # Errors
/// If a internal call set a errno (I/O OS error), an error variant will be returned.
#[inline]
pub fn mkdtemp(template: &str) -> io::Result<String> {
    let mut template_cstr = {
        let mut t = String::new();
        t.push_str(template);
        t.push('\0');
        t
    };

    let ptr = unsafe {
        libc::mkdtemp(template_cstr.as_mut_ptr() as *mut libc::c_char) as *const libc::c_char
    };

    if ptr.is_null() {
        Err(IOError::last_os_error())
    } else {
        template_cstr.pop();
        Ok(template_cstr)
    }
}
