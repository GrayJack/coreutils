use clap::ArgMatches;

/// Represents the command line arguments available to `ls`
#[derive(Default, Copy, Clone)]
pub(crate) struct Flags {
    pub all: bool,
    pub almost_all: bool,
    pub block_size: bool,
    pub classify: bool,
    pub comma_separate: bool,
    pub dereference: bool,
    pub file_status_modification: bool,
    pub indicator: bool,
    pub inode: bool,
    pub last_accessed: bool,
    pub list: bool,
    pub no_owner: bool,
    pub no_sort: bool,
    pub numeric_uid_gid: bool,
    pub order_left_to_right: bool,
    pub order_top_to_bottom: bool,
    pub reverse: bool,
    pub size: bool,
    pub sort_size: bool,
    pub time: bool,
}

impl Flags {
    /// Create a `Flags` instance from the parsed command line arguments
    pub fn from_matches(matches: &ArgMatches<'_>) -> Self {
        let all = matches.is_present("all");
        let almost_all = matches.is_present("almost_all");
        let block_size = matches.is_present("block_size");
        let classify = matches.is_present("classify");
        let comma_separate = matches.is_present("comma_separate");
        let dereference = matches.is_present("dereference");
        let file_status_modification = matches.is_present("file_status_modification");
        let indicator = matches.is_present("indicator");
        let inode = matches.is_present("inode");
        let last_accessed = matches.is_present("last_accessed");
        let list = matches.is_present("list");
        let no_owner = matches.is_present("no_owner");
        let no_sort = matches.is_present("no_sort");
        let numeric_uid_gid = matches.is_present("numeric_uid_gid");
        let order_left_to_right = matches.is_present("order_left_to_right");
        let order_top_to_bottom = matches.is_present("order_top_to_bottom");
        let reverse = matches.is_present("reverse");
        let size = matches.is_present("size");
        let sort_size = matches.is_present("sort_size");
        let time = matches.is_present("time");

        Flags {
            all,
            almost_all,
            block_size,
            classify,
            comma_separate,
            dereference,
            file_status_modification,
            inode,
            indicator,
            last_accessed,
            list,
            no_owner,
            no_sort,
            numeric_uid_gid,
            order_left_to_right,
            order_top_to_bottom,
            reverse,
            size,
            sort_size,
            time,
        }
    }

    /// Whether to print as a list based ont the provided flags
    pub fn show_list(&self) -> bool {
        !(self.comma_separate || self.order_left_to_right || self.order_top_to_bottom)
            && (self.list || self.no_owner || self.numeric_uid_gid)
    }

    /// Whether or not to show hidden files and directories
    pub fn show_hidden(&self) -> bool { self.all || self.almost_all || self.no_sort }
}
