//! Module for TTY abstractions.

use std::{
    error::Error as StdError,
    ffi::CStr,
    fmt::{self, Display},
    os::unix::io::AsRawFd,
};

// use crate::file_descriptor::FileDescriptor;
use bstr::{BStr, BString, ByteSlice};
use libc::{ioctl, ttyname, winsize, TIOCGWINSZ};

/// Possible errors while trying to get a TTY name
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Error {
    /// Not a TTY error
    NotTty,
    /// Any other error
    LibcCall(String, i32),
}

impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotTty => write!(f, "Not a TTY"),
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
pub struct TtyName(BString);

impl TtyName {
    /// Create a [`TTYName`] from a `file_descriptor`
    ///
    /// # Errors
    /// It returns a error variant when `file_descriptor` is not a TTY.
    #[inline]
    pub fn new(file_descriptor: &impl AsRawFd) -> Result<Self, Error> {
        let name = unsafe { ttyname(file_descriptor.as_raw_fd()) };

        let name = if name.is_null() {
            return Err(Error::NotTty);
        } else {
            let name_cstr = unsafe { CStr::from_ptr(name) };
            BString::from(name_cstr.to_bytes())
        };

        Ok(TtyName(name))
    }

    /// Extracts a bstring slice containing the entire [`BString`].
    #[inline]
    pub fn as_bstr(&self) -> &BStr {
        self.0.as_bstr()
    }
}

impl Display for TtyName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// Convenience trait to use [`is_tty`] function as method
pub trait IsTty: AsRawFd {
    /// Check if caller is a TTY.
    ///
    /// ## Example
    /// ```rust
    /// # use coreutils_core::os::tty::IsTty;
    /// # use std::io;
    /// let istty = io::stdin().is_tty();
    /// ```
    fn is_tty(&self) -> bool;
}

impl<T: AsRawFd> IsTty for T {
    #[inline]
    fn is_tty(&self) -> bool {
        is_tty(self)
    }
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

/// Gets the width and height of a TTY.
///
/// ## Example
/// ``` rust
/// use coreutils_core::os::tty::tty_dimensions;
/// let dimensions = tty_dimensions(&std::io::stdout());
/// ```
#[inline]
pub fn tty_dimensions(file_descriptor: &impl AsRawFd) -> Option<(u16, u16)> {
    if !is_tty(file_descriptor) {
        return None;
    }

    let mut size = winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };

    let tiocgwinsz = TIOCGWINSZ;

    #[cfg(target_os = "freebsd")]
    let tiocgwinsz: u64 = tiocgwinsz.into();

    if unsafe { ioctl(file_descriptor.as_raw_fd(), tiocgwinsz, &mut size) } == -1 {
        return None;
    }

    Some((size.ws_col, size.ws_row))
}
