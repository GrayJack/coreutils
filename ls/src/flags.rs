use clap::ArgMatches;

#[derive(Default, Copy, Clone)]
pub(crate) struct LsFlags {
    pub all: bool,
    pub classify: bool,
    pub comma_separate: bool,
    pub list: bool,
    pub no_owner: bool,
    pub numeric_uid_gid: bool,
    pub reverse: bool,
    pub size: bool,
    pub time: bool,
}

impl LsFlags {
    pub fn from_matches(matches: &ArgMatches<'_>) -> Self {
        let all = matches.is_present("all");
        let classify = matches.is_present("classify");
        let comma_separate = matches.is_present("comma_separate");
        let list = matches.is_present("list");
        let no_owner = matches.is_present("no_owner");
        let numeric_uid_gid = matches.is_present("numeric_uid_gid");
        let reverse = matches.is_present("reverse");
        let size = matches.is_present("size");
        let time = matches.is_present("time");

        LsFlags {
            all,
            classify,
            comma_separate,
            list,
            no_owner,
            numeric_uid_gid,
            reverse,
            size,
            time,
        }
    }

    /// Whether to print as a list based ont the provided flags
    pub fn show_list(&self) -> bool {
        !self.comma_separate && self.list || self.no_owner || self.numeric_uid_gid
    }
}
