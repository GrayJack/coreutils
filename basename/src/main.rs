use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("basename.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let multiple_paths = matches.is_present("multiple") || matches.is_present("suffix");

    let suffix = if matches.is_present("suffix") {
        // Using unwrap here is ok since we already checked if --suffix/-s was used or not.
        matches.value_of("suffix").unwrap()
    } else {
        ""
    };

    let line_ending = if matches.is_present("zero") { '\0' } else { '\n' };

    if !multiple_paths {
        let path = matches.value_of("NAME").unwrap();
        print!("{} {}", basename(path, suffix), line_ending);
    } else {
        let paths = matches.values_of("NAME").unwrap();
        for path in paths {
            print!("{} {}", basename(path, suffix), line_ending);
        }
    }
}

/// Get `full_path` basename, removing the given `suffix`.
/// **Examples:**
/// ```rust
/// let name = basename("~/Pictures/mypicture.jpg", "");
/// assert_eq!("mypicture.jpg".to_string(), name);
/// ```
///
/// ```rust
/// let name = basename("~/Pictures/mypicture.jpg", ".jpg");
/// assert_eq!("mypicture".to_string(), name);
/// ```
fn basename(full_path: &str, suffix: &str) -> String {
    let split_full_path: Vec<&str> = full_path.split('/').collect();
    match split_full_path.last() {
        Some(name) => strip_suffix(name, suffix),
        None => "".to_owned(),
    }
}

/// Removes the given `suffix` from the `name`.
fn strip_suffix(name: &str, suffix: &str) -> String {
    if name == suffix {
        return name.to_owned();
    }
    if name.ends_with(suffix) {
        return name[..(name.len() - suffix.len())].to_owned();
    }
    name.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basename_empty_suffix_file() {
        assert_eq!("image.jpg".to_string(), basename("~/Pictures/image.jpg", ""));
        assert_eq!("doc.pdf".to_string(), basename("~/Documents/doc.pdf", ""));
    }

    #[test]
    fn basename_suffix_file() {
        assert_eq!("image".to_string(), basename("~/Pictures/image.jpg", ".jpg"));
        assert_eq!("doc".to_string(), basename("~/Documents/doc.pdf", ".pdf"));
    }

    #[test]
    fn basename_empty_suffix_dir() {
        assert_eq!("bin", basename("/usr/bin", ""));
        assert_eq!("Documents", basename("~/Documents", ""));
    }

    #[test]
    fn basename_suffix_dir() {
        assert_eq!("b", basename("/usr/bin", "in"));
        assert_eq!("Doc", basename("~/Documents", "uments"));
    }
}
