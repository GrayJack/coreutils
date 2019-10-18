// Re-exports
pub use bstr::{self, BStr, BString, ByteSlice, ByteVec, B};
pub use time;

// Agnostic Modules
pub mod consts;
pub mod env;
pub mod file_descriptor;
pub mod group;
pub mod passwd;
pub mod tty;
pub mod types;
pub mod utsname;

// Specific Modules
#[cfg(not(target_os = "fuchsia"))]
pub mod priority;

// Remove after libc supports utmpx for these plataforms
#[cfg(not(any(target_os = "netbsd", target_os = "solaris")))]
#[cfg(not(any(target_os = "fuchsia", target_os = "haiku", target_os = "openbsd")))]
pub mod utmpx;

#[cfg(any(target_os = "freebsd", target_os = "macos"))]
pub mod audit;

#[cfg(target_os = "openbsd")]
pub mod routing_table;
