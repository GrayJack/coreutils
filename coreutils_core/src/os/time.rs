//! Module for time related abstractions more close to the OS.
#[cfg(not(target_os = "haiku"))]
use std::ptr;
use std::{io, mem::MaybeUninit};

use libc::localtime_r;

use super::{Time, TimeVal, Tm};

/// Set the system time as `timeval`
///
/// # Errors
/// If a internal call set a errno (I/O OS error), an error variant will be returned.
#[cfg(not(target_os = "haiku"))]
#[inline]
pub fn set_time_of_day(timeval: TimeVal) -> io::Result<()> {
    let result = unsafe { libc::settimeofday(&timeval as *const TimeVal, ptr::null()) };
    match result {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}

/// Get the time structure with local time offset.
///
/// # Errors
/// If a internal call set a errno (I/O OS error), an error variant will be returned.
#[inline]
pub fn local_time(timestamp: i64) -> io::Result<Tm> {
    // We do this cause libc doesn't expose this function in solarish
    // This way we aboid conditional compilation
    // TODO(GrayJack): Do a PR to libc to include tzset into solarish platforms
    extern "C" {
        fn tzset();
    }

    let mut tm = MaybeUninit::uninit();

    unsafe { tzset() };

    let tm_ptr = unsafe { localtime_r(&(timestamp as Time), tm.as_mut_ptr()) };

    if tm_ptr.is_null() { Err(io::Error::last_os_error()) } else { Ok(unsafe { tm.assume_init() }) }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Io(io::Error),
    Time(std::time::SystemTimeError),
    TargetNotSupported,
}

impl std::error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Time(err) => Some(err),
            Self::TargetNotSupported => None,
        }
    }
}

impl std::fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{}", err),
            Self::Time(err) => write!(f, "{}", err),
            Self::TargetNotSupported => write!(f, "This platform are not supported"),
        }
    }
}

impl From<io::Error> for Error {
    #[inline]
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<std::time::SystemTimeError> for Error {
    #[inline]
    fn from(err: std::time::SystemTimeError) -> Self {
        Self::Time(err)
    }
}

/// Get the time the system started.
#[cfg(not(target_os = "fuchsia"))]
#[cfg_attr(feature = "inline-more", inline)]
pub fn boottime() -> Result<TimeVal, Error> {
    #[cfg(not(any(target_os = "solaris", target_os = "illumos")))]
    let mut bootime = TimeVal { tv_sec: 0, tv_usec: 0 };

    #[cfg(target_os = "linux")]
    {
        use std::time::SystemTime;

        let string = std::fs::read_to_string("/proc/uptime")?;
        let mut secs =
            string.trim().split_whitespace().take(2).filter_map(|val| val.parse::<f64>().ok());
        let sec = secs.next().unwrap() as libc::time_t;
        let micro = secs.next().unwrap() as libc::suseconds_t;

        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

        bootime.tv_sec = now.as_secs() as libc::time_t - sec;
        bootime.tv_usec = micro;

        Ok(bootime)
    }

    #[cfg(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "macos"
    ))]
    {
        static CTL_KERN: libc::c_int = 1;
        static KERN_BOOTTIME: libc::c_int = 21;

        let mut syscall = [CTL_KERN, KERN_BOOTTIME];
        let mut size: libc::size_t = std::mem::size_of_val(&bootime) as libc::size_t;
        let res = unsafe {
            libc::sysctl(
                syscall.as_mut_ptr(),
                2,
                &mut bootime as *mut libc::timeval as *mut libc::c_void,
                &mut size,
                ptr::null_mut(),
                0,
            )
        };

        match res {
            0 => Ok(bootime),
            _ => Err(Error::Io(io::Error::last_os_error())),
        }
    }

    #[cfg(target_os = "haiku")]
    {
        Err(Error::TargetNotSupported)
    }

    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    {
        Err(Error::TargetNotSupported)
    }
}

/// Get the time the system is up since boot.
#[cfg(not(target_os = "fuchsia"))]
#[cfg_attr(feature = "inline-more", inline)]
pub fn uptime() -> Result<TimeVal, Error> {
    #[cfg(not(any(target_os = "solaris", target_os = "illumos")))]
    let mut uptime = TimeVal { tv_sec: 0, tv_usec: 0 };

    #[cfg(target_os = "linux")]
    {
        let string = std::fs::read_to_string("/proc/uptime")?;
        let mut secs =
            string.trim().split_whitespace().take(2).filter_map(|val| val.parse::<f64>().ok());
        uptime.tv_sec = secs.next().unwrap() as libc::time_t;
        uptime.tv_usec = secs.next().unwrap() as libc::suseconds_t;

        Ok(uptime)
    }

    #[cfg(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "macos"
    ))]
    {
        use std::time::SystemTime;

        static CTL_KERN: libc::c_int = 1;
        static KERN_BOOTTIME: libc::c_int = 21;

        let mut syscall = [CTL_KERN, KERN_BOOTTIME];
        let mut size: libc::size_t = std::mem::size_of_val(&uptime) as libc::size_t;
        let res = unsafe {
            libc::sysctl(
                syscall.as_mut_ptr(),
                2,
                &mut uptime as *mut libc::timeval as *mut libc::c_void,
                &mut size,
                ptr::null_mut(),
                0,
            )
        };

        match res {
            0 => {
                let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
                let now = now.as_secs();

                uptime.tv_sec = now as i64 - uptime.tv_sec;
                Ok(uptime)
            },
            _ => Err(Error::Io(io::Error::last_os_error())),
        }
    }

    #[cfg(target_os = "haiku")]
    {
        use libc::system_time;

        const USECS_PER_SEC: i64 = 1_000_000;
        // Not sure how safe it is, but all usages that I(GrayJack) saw on C and C++ code didn't
        // handle no error cases
        let uptime_usecs = unsafe { system_time() };
        let mut uptime_secs = uptime_usecs / USECS_PER_SEC;
        let uptime_usecs = uptime_usecs % USECS_PER_SEC;
        if uptime_usecs >= (USECS_PER_SEC / 2) {
            uptime_secs += 1;
        }

        uptime.tv_sec = uptime_secs;
        uptime.tv_usec = uptime_usecs as _;

        Ok(uptime)
    }

    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    {
        Err(Error::TargetNotSupported)
    }
}
