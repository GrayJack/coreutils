//! Module for time related abstractions more close to the OS.
use std::{io, mem::MaybeUninit, ptr};

use libc::localtime_r;

use super::{Time, TimeVal, Tm};


/// Set the system time as `timeval`
///
/// # Errors
/// If a internal call set a errno (I/O OS error), an error variant will be returned.
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

/// Get the time the system is up since boot
pub fn uptime() -> io::Result<TimeVal> {
    let mut boot_time = TimeVal { tv_sec: 0, tv_usec: 0 };

    #[cfg(target_os = "linux")]
    {
        let string = std::fs::read_to_string("/proc/uptime")?.replace(".", "");
        let mut secs =
            string.trim().split_whitespace().take(2).filter_map(|val| val.parse::<f64>().ok());
        boot_time.tv_sec = secs.next().unwrap() as libc::time_t;
        boot_time.tv_usec = secs.next().unwrap() as libc::suseconds_t;

        Ok(boot_time)
    }

    #[cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "macos"
    ))]
    {
        static CTL_KERN: libc::c_int = 1;
        static KERN_BOOTTIME: libc::c_int = 21;

        let mut syscall = [CTL_KERN, KERN_BOOTTIME];
        let mut size: libc::size_t = std::mem::size_of_val(&boot_time) as libc::size_t;
        let res = unsafe {
            libc::sysctl(
                syscall.as_mut_ptr(),
                2,
                &mut boot_time as *mut libc::timeval as *mut libc::c_void,
                &mut size,
                ptr::null_mut(),
                0,
            )
        };

        match res {
            0 => Ok(boot_time),
            _ => Err(io::Error::last_os_error()),
        }
    }
    // #[cfg(target_os = "solaris")]
    // {
    //     let start = kstat::boot_time()?;
    //     let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    //     let now = now.as_secs();
    //     if now < start {
    //         return Err(Error::General("time went backwards".into()));
    //     }
    //     boot_time.tv_sec = (now - start) as i64;
    // }

    // Ok(boot_time)
}
