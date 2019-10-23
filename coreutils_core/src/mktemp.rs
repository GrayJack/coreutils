//! Module with wrappers for libc mkstemp(3), mkdtemp(3)

use std::{
    fmt::{self, Display},
    fs::File,
    io::Error,
    os::unix::io::FromRawFd,
};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MktempError {
    err: String,
}

/// A struct that represents a mktemp(3) result.
/// This includes the file created, and the path to that file.
#[derive(Debug)]
pub struct Mktemp {
    pub file: File,
    pub path: String,
}

impl Display for Mktemp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.path) }
}

impl Display for MktempError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.err) }
}

/// Creates a temporary file based on the given template.
///
/// The template should end with a number of `X` characters.
///
/// Some libc implementations requires the template to end with a minimum number of X
/// characters (for example, 6 in glibc as of version 2.29).
pub fn mkstemp(template: &str) -> Result<Mktemp, MktempError> {
    let mut template_cstr = {
        let mut t = String::new();
        t.push_str(template);
        t.push('\0');
        t
    };
    let fd = unsafe { libc::mkstemp(template_cstr.as_mut_ptr() as *mut libc::c_char) };
    if fd == -1 {
        let error_str = match Error::last_os_error().raw_os_error().unwrap() {
            22 => "Too few X's in template".to_string(), // EINVAL
            _ => Error::last_os_error().to_string(),     /* error from stat(2) (BSD) or open(2)
                                                           * (glibc) */
        };
        return Err(MktempError { err: error_str });
    }

    // remove the trailing \0
    template_cstr.pop();

    Ok(Mktemp { file: unsafe { File::from_raw_fd(fd) }, path: template_cstr })
}

/// Creates a temporary directory based on the given template.
///
/// The template should end with a number of `X` characters.
///
/// Some libc implementations requires the template to end with a minimum number of X
/// characters (for example, 6 in glibc as of version 2.29).
pub fn mkdtemp(template: &str) -> Result<String, MktempError> {
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
        let error_str = match Error::last_os_error().raw_os_error().unwrap() {
            22 => "Too few X's in template".to_string(), // EINVAL
            _ => Error::last_os_error().to_string(),     /* error from stat(2) (BSD) or open(2)
                                                           * (glibc) */
        };
        Err(MktempError { err: error_str })
    } else {
        template_cstr.pop();
        Ok(template_cstr)
    }
}
