#[cfg(target_family = "unix")]
use std::os::unix::fs::MetadataExt;
use std::{fs::Metadata, path::Display, process};

use clap::{load_yaml, App, AppSettings::ColoredHelp, ArgMatches};
use glob::Pattern;
use walkdir::WalkDir;

mod blocksize;
mod time;

use blocksize::{Blocksize, BlocksizeError};
use time::{DuTime, TimeOption, TimeStyleOption};

#[cfg(test)]
mod tests;

fn main() {
    let yaml = load_yaml!("du.yml");
    let matches = App::from_yaml(yaml)
        .settings(&[ColoredHelp])
        .help_message("Display help information")
        .version_message("Display version information")
        .get_matches();

    let flags = DuFlagsAndOptions::from_matches(&matches);
    let paths = parse_files(&matches);

    let mut grand_total = 0;

    for path in paths {
        process_path(path, &flags, &mut grand_total);
    }

    if flags.grand_total {
        let total_value = if flags.use_inodes {
            DisplayValue::INodes(grand_total)
        } else {
            DisplayValue::DiskUsage(Blocksize::new().with_value(grand_total))
        };
        print_du(total_value, String::from("total"), &flags);
    }
}

#[derive(Debug)]
enum DisplayValue {
    INodes(u64),
    DiskUsage(Blocksize),
}

impl DisplayValue {
    fn size(&self) -> u64 {
        match &self {
            DisplayValue::INodes(inodes) => *inodes,
            DisplayValue::DiskUsage(blocksize) => blocksize.value(),
        }
    }
}

#[derive(Debug)]
struct DuFlagsAndOptions<'a> {
    pub show_all: bool,
    pub use_apparent_size: bool,
    pub count_links: bool,
    pub dereference: bool,
    pub dereference_args: bool,
    pub print_human_readable: bool,
    pub use_si: bool,
    pub use_inodes: bool,
    pub use_ascii_null: bool,
    pub one_file_system: bool,
    pub seperate_dirs: bool,
    pub grand_total: bool,
    pub blocksize: Blocksize,
    pub exclude_pattern: Option<Pattern>,
    pub max_depth: Option<usize>,
    pub threshold: Option<(bool, Blocksize)>,
    pub time: Option<TimeOption>,
    pub time_style: TimeStyleOption<'a>,
}

impl<'a> DuFlagsAndOptions<'a> {
    pub fn from_matches(matches: &'a ArgMatches) -> Self {
        let def_index = matches.index_of("dereference").unwrap_or(0);
        let no_def_index = matches.index_of("no-dereference").unwrap_or(0);

        DuFlagsAndOptions {
            show_all: matches.is_present("all"),
            use_apparent_size: matches.is_present("apparent-size") || matches.is_present("bytes"),
            count_links: matches.is_present("count-links"),
            dereference: if def_index > no_def_index {
                matches.is_present("dereference")
            } else {
                matches.is_present("no-dereference")
            },
            dereference_args: matches.is_present("dereference-args")
                || matches.is_present("dereference-args-alias"),
            print_human_readable: matches.is_present("human-readable")
                || matches.value_of("block-size").unwrap_or("") == "human-readable",
            use_si: matches.is_present("si")
                || matches.value_of("block-size").unwrap_or("") == "si",
            use_inodes: matches.is_present("inodes"),
            use_ascii_null: matches.is_present("line-end-null"),
            one_file_system: matches.is_present("one-file-system"),
            seperate_dirs: matches.is_present("seperate-dirs"),
            grand_total: matches.is_present("total"),
            blocksize: parse_blocksize(matches),
            exclude_pattern: parse_exclude_pattern(matches.value_of("exclude-pattern")),
            max_depth: parse_depth(matches),
            threshold: parse_threshold(matches.value_of("threshold")),
            time: parse_time(matches),
            time_style: parse_time_style(matches.value_of("time-style")),
        }
    }
}

fn parse_files<'a>(matches: &'a ArgMatches) -> Vec<&'a str> {
    if let Some(files) = matches.values_of("FILE") {
        return files.collect();
    }
    vec!["."]
}

fn parse_blocksize(matches: &ArgMatches) -> Blocksize {
    let initial_size = Blocksize::new();

    if matches.is_present("human-readable") {
        return initial_size.with_value(1024);
    }

    if matches.is_present("si") {
        return initial_size.with_value(1000);
    }

    if matches.is_present("bytes") {
        return initial_size.with_value(1);
    }

    if matches.is_present("block-size-k") {
        match initial_size.with_value(1).with_suffix("K") {
            Ok(blocksize) => return blocksize,
            _ => {
                process::exit(1);
            },
        }
    }

    if matches.is_present("block-size-m") {
        match initial_size.with_value(1).with_suffix("M") {
            Ok(blocksize) => return blocksize,
            _ => {
                process::exit(1);
            },
        }
    }

    if let Some(size) = matches.value_of("block-size") {
        match Blocksize::from_str(size) {
            Ok(blocksize) => {
                return blocksize;
            },
            Err(err) => {
                match err {
                    BlocksizeError::InvalidBlocksize => {
                        eprintln!("du: invalid --block-size argument: '{}'", size);
                    },
                    BlocksizeError::InvalidSuffixError(s) => {
                        eprintln!("du: invalid suffix in --block-size argument: '{}'", &s)
                    },
                }
                process::exit(1);
            },
        }
    }
    initial_size
}

fn parse_exclude_pattern(value: Option<&str>) -> Option<Pattern> {
    if let Some(pattern) = value {
        match Pattern::new(pattern) {
            Ok(p) => return Some(p),
            Err(err) => {
                println!("du: error parsing value for --exclude-pattern: {}", err);
                process::exit(1);
            },
        }
    }
    None
}

fn parse_depth(matches: &ArgMatches) -> Option<usize> {
    if matches.is_present("summarize") {
        return Some(0);
    }
    if let Some(depth) = matches.value_of("max-depth") {
        match depth.parse::<usize>() {
            Ok(number) => return Some(number),
            Err(err) => {
                eprintln!("du: error parsing value for --max-depth: {}", err);
                process::exit(1);
            },
        }
    }
    None
}

fn parse_threshold(value: Option<&str>) -> Option<(bool, Blocksize)> {
    if let Some(threshold) = value {
        let is_negative = threshold.starts_with("-");

        let threshold_slice = if is_negative { &threshold[1..] } else { threshold };

        match Blocksize::from_str(threshold_slice) {
            Ok(blocksize) => return Some((is_negative, blocksize)),
            Err(err) => match err {
                BlocksizeError::InvalidBlocksize => {
                    eprintln!("du: invalid --threshold argument: '{}'", threshold);
                },
                BlocksizeError::InvalidSuffixError(s) => {
                    eprintln!("du: invalid suffix in --threshold argument: '{}'", &s);
                    process::exit(1);
                },
            },
        }
    }
    None
}

fn parse_time(matches: &ArgMatches) -> Option<TimeOption> {
    if matches.is_present("time") {
        // unwrap safe because of `default_value: mtime`
        let time = matches.value_of("time").unwrap();
        match time {
            "mtime" => {
                return Some(TimeOption::MTime);
            },
            "atime" | "access" => {
                return Some(TimeOption::ATime);
            },
            "ctime" | "status" | "use" => {
                return Some(TimeOption::CTime);
            },
            _ => {
                eprintln!("du: invalid --time argument: {}", time);
                process::exit(1);
            },
        }
    }
    None
}

fn parse_time_style(value: Option<&str>) -> TimeStyleOption {
    if let Some(style) = value {
        if style.starts_with("+") {
            let f = match style.chars().skip(1).next() {
                Some(_) => &style[1..],
                None => "",
            };
            return TimeStyleOption::Format(f);
        }
        match style {
            "full-iso" => return TimeStyleOption::FullIso,
            "long-iso" | "" => return TimeStyleOption::LongIso,
            "iso" => return TimeStyleOption::Iso,
            _ => {
                eprintln!("du: invalid --time-style argument: {}", &style);
                process::exit(1);
            },
        };
    } else {
        return TimeStyleOption::LongIso;
    }
}

fn process_path(path: &str, flags_opts: &DuFlagsAndOptions, total_ref: &mut u64) {
    let walker = WalkDir::new(path)
        .same_file_system(flags_opts.one_file_system)
        .follow_links(flags_opts.dereference)
        .contents_first(true);

    // tracks depth
    let mut current_depth = 0;

    // tracks subdir sizes, indexed by depth [0] => root
    let mut subdir_sizes = vec![0_u64];

    // tracks max m/a/c-time of a subdir
    let mut subir_max_times = vec![DuTime::new(0)];

    let mut arg_total = 0;

    walker
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            flags_opts.exclude_pattern.as_ref().map_or(true, |p| !p.matches_path(&entry.path()))
        })
        .for_each(|entry| {
            current_depth = entry.depth();
            if let Some(meta) = entry.metadata().ok() {
                if meta.is_dir() {
                    let value =
                        process_value(&meta, flags_opts, &mut subdir_sizes, current_depth, true);

                    if flags_opts.grand_total {
                        arg_total = value.size();
                    }

                    if let Some(t) = &flags_opts.time {
                        let time = process_time(&meta, t, &mut subir_max_times, current_depth);
                        filter_and_print(
                            path,
                            &entry.path().display(),
                            value,
                            Some(time),
                            flags_opts,
                            current_depth,
                            true,
                        );
                    } else {
                        filter_and_print(
                            path,
                            &entry.path().display(),
                            value,
                            None,
                            flags_opts,
                            current_depth,
                            true,
                        );
                    }
                } else {
                    let value =
                        process_value(&meta, flags_opts, &mut subdir_sizes, current_depth, false);

                    if let Some(t) = &flags_opts.time {
                        let time = process_time(&meta, t, &mut subir_max_times, current_depth);
                        filter_and_print(
                            path,
                            &entry.path().display(),
                            value,
                            Some(time),
                            flags_opts,
                            current_depth,
                            false,
                        );
                    } else {
                        filter_and_print(
                            path,
                            &entry.path().display(),
                            value,
                            None,
                            flags_opts,
                            current_depth,
                            false,
                        );
                    }
                }
            }
        });

    if flags_opts.grand_total {
        *total_ref += arg_total;
    }
}

// returns file size and manages the subdir sizes vector
fn process_value(
    meta: &Metadata, flags_opts: &DuFlagsAndOptions, subdir_sizes_r: &mut Vec<u64>, depth: usize,
    is_dir: bool,
) -> DisplayValue
{
    let subdir_count = subdir_sizes_r.len() - 1;

    if depth > subdir_count {
        // fill up vec, so index equals depth. e.g. file depth = 2 => [0, 0 ,0]
        for _ in 0..(depth - subdir_count) {
            subdir_sizes_r.push(0);
        }
    }

    let display_value = get_display_value(meta, flags_opts);

    if !is_dir || !flags_opts.seperate_dirs {
        // add size to subdir total size
        subdir_sizes_r[depth] += display_value.size();
    }

    if is_dir && depth < subdir_count {
        // when recursing back to the parent directory
        let subdir_sum = subdir_sizes_r.pop().unwrap_or(0);

        if !flags_opts.seperate_dirs {
            subdir_sizes_r[depth] += subdir_sum;
        }

        match display_value {
            DisplayValue::INodes(i) => return DisplayValue::INodes(i + subdir_sum),
            DisplayValue::DiskUsage(b) => {
                let blk_val = b.value();
                return DisplayValue::DiskUsage(b.with_value(blk_val + subdir_sum));
            },
        }
    }

    display_value
}

fn get_display_value(metadata: &Metadata, flags_opts: &DuFlagsAndOptions) -> DisplayValue {
    if flags_opts.use_inodes {
        return DisplayValue::INodes(get_inode());
    }

    let bytes = get_bytes(metadata, flags_opts.use_apparent_size);

    let mut disk_usage = Blocksize::new().with_value(bytes);

    if flags_opts.use_si {
        disk_usage.use_si();
    }

    DisplayValue::DiskUsage(disk_usage)
}

#[cfg(target_family = "unix")]
fn get_bytes(metadata: &Metadata, use_apparent_size: bool) -> u64 {
    if use_apparent_size {
        metadata.len()
    } else {
        // returns 512 byte untis
        // https://doc.rust-lang.org/src/std/os/linux/fs.rs.html#308
        metadata.blocks() * 512
    }
}

#[cfg(not(target_family = "unix"))]
fn get_bytes(metadata: &Metadata, use_apparent_size: bool) -> u64 { metadata.len() }

#[cfg(target_family = "unix")]
fn get_inode() -> u64 { 1 }

#[cfg(not(target_family = "unix"))]
fn get_inode() -> u64 {
    eprintln!("du: unix only");
    process::exit(1);
}

// returns file time and manages the max_time vector
fn process_time(
    meta: &Metadata, time: &TimeOption, subdir_times_r: &mut Vec<DuTime>, depth: usize,
) -> DuTime {
    let subdir_times_count = subdir_times_r.len() - 1;

    if depth > subdir_times_count {
        for _ in 0..(depth - subdir_times_count) {
            subdir_times_r.push(DuTime::new(0));
        }
    }

    let display_time = get_display_time(&meta, &time);

    if display_time > subdir_times_r[depth] {
        // replace subdir max value
        subdir_times_r[depth] = display_time.clone();
    }

    display_time
}

#[cfg(target_family = "unix")]
fn get_display_time(metadata: &Metadata, time: &TimeOption) -> DuTime {
    match time {
        TimeOption::ATime => DuTime::new(metadata.atime()).with_nano_seconds(metadata.atime_nsec()),
        TimeOption::CTime => DuTime::new(metadata.ctime()).with_nano_seconds(metadata.ctime_nsec()),
        TimeOption::MTime => DuTime::new(metadata.mtime()).with_nano_seconds(metadata.mtime_nsec()),
    }
}

#[cfg(not(target_family = "unix"))]
fn get_display_time(metadata: &Metadata, time: &TimeOption) -> DuTime {
    match time {
        TimeOption::ATime => DuTime::new(get_sec(metadata.accessed()).unwrap_or(0)),
        TimeOption::CTime => DuTime::new(get_sec(metadata.created()).unwrap_or(0)),
        TimeOption::MTime => DuTime::new(get_sec(metadata.modified()).unwrap_or(0)),
    }
}

#[cfg(not(target_family = "unix"))]
fn get_sec(sys_time_res: Result<SystemTime, Error>) -> Option<i64> {
    match sys_time_res {
        Ok(sys_time) => match sys_time.duration_since(UNIX_EPOCH) {
            Ok(duration) => Some(duration.as_secs() as i64),
            Err(_err) => None,
        },
        Err(_err) => None,
    }
}

// applies filters from args before printing
fn filter_and_print(
    root: &str, path: &Display, value: DisplayValue, time: Option<DuTime>,
    flags_opts: &DuFlagsAndOptions, depth: usize, is_dir: bool,
)
{
    let print_entry: bool;

    if is_dir {
        print_entry = satisfies_threshold(&value, &flags_opts.threshold)
            && flags_opts.max_depth.map_or(true, |max| depth <= max)
    } else {
        print_entry = (flags_opts.show_all || path.to_string() == root)
            && satisfies_threshold(&value, &flags_opts.threshold)
            && flags_opts.max_depth.map_or(true, |max| depth <= max)
    }

    if print_entry {
        if let Some(t) = time {
            print_du_with_time(value, t, path.to_string(), flags_opts);
        } else {
            print_du(value, path.to_string(), flags_opts);
        }
    }
}

fn satisfies_threshold(value: &DisplayValue, threshold_opt: &Option<(bool, Blocksize)>) -> bool {
    match value {
        DisplayValue::INodes(_i) => true,
        DisplayValue::DiskUsage(blocksize) => {
            if let Some(threshold) = threshold_opt {
                let (t_is_negative, t_value) = (threshold.0, threshold.1.value());

                // exclude entries greater than THRESHOLD if negative
                if t_is_negative && blocksize.value() > t_value {
                    return false;
                }

                // exclude entries smaller then THRESHOLD if positive
                if !t_is_negative && blocksize.value() < t_value {
                    return false;
                }
            }
            true
        },
    }
}

fn print_du(value: DisplayValue, path: String, flags_opts: &DuFlagsAndOptions) {
    if satisfies_threshold(&value, &flags_opts.threshold) {
        print!(
            "{}\t{}{}",
            format_display_value(value, flags_opts),
            path,
            if flags_opts.use_ascii_null { "\0" } else { "\n" }
        );
    }
}

fn print_du_with_time(
    value: DisplayValue, time: DuTime, path: String, flags_opts: &DuFlagsAndOptions,
) {
    if satisfies_threshold(&value, &flags_opts.threshold) {
        print!(
            "{}\t{}{}{}",
            format_display_value(value, flags_opts),
            format_display_time(time, flags_opts),
            path,
            if flags_opts.use_ascii_null { "\0" } else { "\n" }
        );
    }
}

fn format_display_value(value: DisplayValue, flags_opts: &DuFlagsAndOptions) -> String {
    match value {
        DisplayValue::INodes(inodes) => inodes.to_string(),
        DisplayValue::DiskUsage(blocksize) => {
            if flags_opts.print_human_readable {
                let with_largest_suffix = blocksize.use_largest_suffix();
                with_largest_suffix.human_readable()
            } else {
                let mut blocksize_fraction = 0;

                if blocksize.value() != 0 && blocksize.value() / flags_opts.blocksize.value() == 0 {
                    blocksize_fraction = 1;
                } else if blocksize.value() != 0 {
                    blocksize_fraction = blocksize.value() / flags_opts.blocksize.value();
                }

                format!("{}{}", blocksize_fraction, flags_opts.blocksize.suffix_str())
            }
        },
    }
}

fn format_display_time(time: DuTime, flags_opts: &DuFlagsAndOptions) -> String {
    time.get_formatted(&flags_opts.time_style)
}
