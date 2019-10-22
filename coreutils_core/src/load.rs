use std::{io, os::raw::c_double};

use libc::getloadavg;

pub fn load_average() -> io::Result<[c_double; 3]> {
    let mut avg: [c_double; 3] = [0.0; 3];
    let res = unsafe { getloadavg(avg.as_mut_ptr(), 3) };

    if res == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(avg)
}
