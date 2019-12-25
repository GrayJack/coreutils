// Re-exports
pub use bstr::{self, BStr, BString, ByteSlice, ByteVec, B};
pub use libc;
pub use time;

// Agnostic Modules
pub mod backup;
pub mod consts;
pub mod env;
pub mod file_descriptor;
pub mod group;
pub mod input;
pub mod mkfifo;
pub mod mktemp;
pub mod passwd;
pub mod process;
pub mod settime;
pub mod tty;
pub mod types;
pub mod utsname;

// Specific Modules
#[cfg(not(any(target_os = "fuchsia", target_os = "haiku")))]
pub mod load;

#[cfg(any(target_os = "netbsd", target_os = "openbsd", target_os = "solaris"))]
pub mod utmp;

#[cfg(not(any(target_os = "fuchsia", target_os = "haiku", target_os = "openbsd")))]
pub mod utmpx;

#[cfg(any(target_os = "freebsd", target_os = "macos"))]
pub mod audit;

#[cfg(target_os = "openbsd")]
pub mod routing_table;
