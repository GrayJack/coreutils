//! Module for TTY abstractions.

use std::{
    error::Error as StdError,
    ffi::CStr,
    fmt::{self, Display},
    os::unix::io::AsRawFd,
};

use libc::ttyname;

// use crate::file_descriptor::FileDescriptor;

use bstr::{BStr, BString, ByteSlice};

/// Possible errors while trying to get a TTY name
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Error {
    /// Not a TTY error
    NotTTY,
    /// Any other error
    LibcCall(String, i32),
}

impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotTTY => write!(f, "Not a TTY"),
            Self::LibcCall(fn_name, err_code) => {
                write!(f, "Failed calling {} with this error code: {}", fn_name, err_code)
            },
        }
    }
}

impl StdError for Error {}

/// A struct that holds the name of a TTY with a [`Display`] trait implementation
/// to be easy to print.
#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct TTYName(BString);

impl TTYName {
    /// Create a [`TTYName`] from a `file_descriptor`
    ///
    /// # Errors
    /// It returns a error variant when `file_descriptor` is not a TTY.
    #[inline]
    pub fn new(file_descriptor: &impl AsRawFd) -> Result<Self, Error> {
        let name = unsafe { ttyname(file_descriptor.as_raw_fd()) };

        let name = if name.is_null() {
            return Err(Error::NotTTY);
        } else {
            let name_cstr = unsafe { CStr::from_ptr(name) };
            BString::from(name_cstr.to_bytes())
        };

        Ok(TTYName(name))
    }

    /// Extracts a bstring slice containing the entire [`BString`].
    #[inline]
    pub fn as_bstr(&self) -> &BStr { self.0.as_bstr() }
}

impl Display for TTYName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { Display::fmt(&self.0, f) }
}

/// Convenience trait to use [`is_tty`] function as method
pub trait IsTTY: AsRawFd {
    /// Check if caller is a TTY.
    ///
    /// ## Example
    /// ```rust
    /// # use coreutils_core::os::tty::IsTTY;
    /// # use std::io;
    /// let istty = io::stdin().is_tty();
    /// ```
    fn is_tty(&self) -> bool;
}

impl<T: AsRawFd> IsTTY for T {
    #[inline]
    fn is_tty(&self) -> bool { is_tty(self) }
}

/// Check if the given `file_descriptor` is a TTY.
///
/// ## Example
/// ```rust
/// use coreutils_core::os::tty::is_tty;
/// let istty = is_tty(&std::io::stdin());
/// ```
#[inline]
pub fn is_tty(file_descriptor: &impl AsRawFd) -> bool {
    unsafe { libc::isatty(file_descriptor.as_raw_fd()) == 1 }
}
