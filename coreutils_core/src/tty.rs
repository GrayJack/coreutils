use std::{
    error::Error as StdError,
    ffi::CStr,
    fmt::{self, Display},
    os::raw::c_int,
};

use libc::ttyname;

use crate::file_descriptor::FileDescriptor;

use bstr::BString;

/// Possible errors while trying to get a TTY name
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Error {
    /// Not a TTY error
    NotTTY,
    /// Any other error
    LibcCall(String, i32),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotTTY => write!(f, "Not a TTY"),
            Self::LibcCall(fn_name, err_code) => write!(
                f,
                "Failed calling {} with this error code: {}",
                fn_name, err_code
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

/// A struct that holds the name of a TTY with a `Display` trait implementation
/// to be easy to print
#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct TTYName(BString);

impl TTYName {
    /// Create a `TTYName` from a `FileDescriptor`
    pub fn new(file_descriptor: FileDescriptor) -> Result<Self, Error> {
        let name = unsafe { ttyname(file_descriptor as c_int) };

        let name = if !name.is_null() {
            let name_cstr = unsafe { CStr::from_ptr(name) };
            BString::from(name_cstr.to_bytes())
        } else {
            return Err(Error::NotTTY);
        };

        Ok(TTYName(name))
    }
}

impl Display for TTYName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Check if the given `FileDescriptor` is a TTY
/// ## Example
/// ```
/// # use coreutils_core::{file_descriptor::FileDescriptor, tty::isatty};
/// # fn main() {
/// let istty = isatty(FileDescriptor::StdIn);
/// # }
/// ```
pub fn isatty(file_descriptor: FileDescriptor) -> bool {
    unsafe { libc::isatty(file_descriptor as c_int) == 1 }
}
