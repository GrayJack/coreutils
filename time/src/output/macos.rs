//! Output interface for `time`

use coreutils_core::os::resource::RUsage;

pub fn default_formatter(_rusage: &RUsage, wall_time: f64, user_time: f64, sys_time: f64) -> String {
    format!("        {:.2} real         {:.2} user         {:.2} sys", wall_time, user_time, sys_time)
}
