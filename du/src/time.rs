use std::cmp::Ordering;

use chrono::NaiveDateTime;

#[derive(Debug, PartialEq)]
pub enum TimeStyleOption<'a> {
    LongIso,
    FullIso,
    Iso,
    Format(&'a str),
}

impl<'a> TimeStyleOption<'a> {
    pub fn get_format(&self) -> &str {
        match self {
            TimeStyleOption::FullIso => "%Y-%m-%d %H:%M:%S.%f",
            TimeStyleOption::LongIso => "%Y-%m-%d %H:%M",
            TimeStyleOption::Iso => "%Y-%m-%d",
            TimeStyleOption::Format(f) => f,
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum TimeOption {
    MTime,
    CTime,
    ATime,
}

#[derive(Clone, Debug, Eq)]
pub struct DuTime {
    seconds: i64,
    n_seconds: i64,
}

impl DuTime {
    pub fn new(seconds: i64) -> DuTime {
        DuTime { seconds, n_seconds: 0 }
    }

    pub fn with_nano_seconds(mut self, n_secs: i64) -> DuTime {
        self.n_seconds = n_secs;
        self
    }

    pub fn get_formatted(&self, style: &TimeStyleOption) -> String {
        let date_time = NaiveDateTime::from_timestamp(self.seconds, self.n_seconds as u32);
        format!("{}\t", date_time.format(style.get_format()))
    }
}

impl Ord for DuTime {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.seconds.cmp(&other.seconds) {
            Ordering::Equal => self.n_seconds.cmp(&other.n_seconds),
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
        }
    }
}

impl PartialOrd for DuTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DuTime {
    fn eq(&self, other: &Self) -> bool {
        self.seconds == other.seconds && self.n_seconds == other.n_seconds
    }
}
