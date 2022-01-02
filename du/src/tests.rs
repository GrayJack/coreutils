use super::*;

#[test]
fn du_parse_files_no_input() {
    assert_eq!(vec!["."], parse_files(&super::cli::create_app().get_matches()));
}

#[test]
fn du_parse_files_multiple_inputs() {
    let m = super::cli::create_app().get_matches_from(vec!["du", "file1", "file2", "file3"]);
    assert_eq!(vec!["file1", "file2", "file3"], parse_files(&m));
}

#[test]
fn du_parse_blocksize_exa() {
    let m = super::cli::create_app().get_matches_from(vec!["du", "-BEiB"]);
    assert_eq!(2u64.pow(60), parse_blocksize(&m).value());
}

#[test]
fn du_parse_blocksize_bytes() {
    let m = super::cli::create_app().get_matches_from(vec!["du", "-b"]);
    assert_eq!(1, parse_blocksize(&m).value());
}

#[test]
fn du_parse_blocksize_human_readable() {
    let m = super::cli::create_app().get_matches_from(vec!["du", "-h"]);
    assert_eq!(1024, parse_blocksize(&m).value());
}

#[test]
fn du_parse_blocksize_si_system() {
    let m = super::cli::create_app().get_matches_from(vec!["du", "-I"]);
    assert_eq!(1000, parse_blocksize(&m).value());
}

#[test]
fn du_parse_time_style_pattern() {
    let year_only_style = "+%Y";
    assert_eq!(TimeStyleOption::Format("%Y"), parse_time_style(Some(year_only_style)));
}
