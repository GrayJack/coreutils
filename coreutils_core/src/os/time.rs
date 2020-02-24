//! Module for time related abstractions more close to the OS.
use std::{io, mem::MaybeUninit, ptr};

use libc::localtime_r;
use time::OffsetDateTime as DateTime;

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

pub fn utc_offset(time: Tm) -> i64 {
    // Not sure if the logic is 100% correct here: All my VMs with solarish systems have a
    // version of Rust below 1.40 with the lastest updates.
    #[cfg(target_os = "solaris")]
    {
        use time::{Date, Time};

        let mut tm = time;
        if tm.tm_sec == 60 {
            // Leap seconds are not currently supported.
            tm.tm_sec = 59;
        }

        let timee = match Time::try_from_hms(tm.tm_hour as u8, tm.tm_min as u8, tm.tm_sec as u8) {
            Ok(t) => t,
            Err(_) => return 0,
        };

        let date = match Date::try_from_yo(1900 + tm.tm_year, tm.tm_yday as u16 + 1) {
            Ok(d) => d,
            Err(_) => return 0,
        };

        let local_timestamp = date.with_time(timee).timestamp();

        local_timestamp - unsafe { libc::mktime(&mut tm) }
    }

    #[cfg(not(target_os = "solaris"))]
    {
        time.tm_gmtoff as i64
    }
}

pub fn system_utc_offset() -> io::Result<i64> {
    let now = DateTime::now();

    Ok(utc_offset(local_time(now.timestamp())?))
}
