//! Module for File descriptor abstractions.

/// A `FileDescriptor` that can be `StdIn`, `StdOut` or `StdErr`
/// Usefull when dealing with C call to `ttyname` and `ttyname_r`
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum FileDescriptor {
    StdIn  = 0,
    StdOut = 1,
    StdErr = 2,
}
