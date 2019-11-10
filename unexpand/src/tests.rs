#![allow(clippy::cognitive_complexity)]
use super::*;

#[test]
fn unexpand_lines() {
    let mut instance = Unexpand { all: false, tabs: TabStops::new(Some("2")).unwrap() };
    assert_eq!(instance.unexpand_line("    c"), "\t\tc\n");
    assert_eq!(instance.unexpand_line("  c"), "\tc\n");
    assert_eq!(instance.unexpand_line("  c  c"), "\tc  c\n");
    assert_eq!(instance.unexpand_line("   c    c"), "\t c    c\n");

    let mut instance = Unexpand { all: true, tabs: TabStops::new(Some("2")).unwrap() };
    assert_eq!(instance.unexpand_line("    c"), "\t\tc\n");
    assert_eq!(instance.unexpand_line("  c"), "\tc\n");
    assert_eq!(instance.unexpand_line("  c  c"), "\tc\tc\n");
    assert_eq!(instance.unexpand_line("   c    c"), "\t c\t\tc\n");

    let mut instance = Unexpand { all: true, tabs: TabStops::new(Some("8")).unwrap() };
    assert_eq!(instance.unexpand_line("    c"), "    c\n");
    assert_eq!(instance.unexpand_line("  c"), "  c\n");
    assert_eq!(instance.unexpand_line("  c  c"), "  c  c\n");
    assert_eq!(instance.unexpand_line("   c    c"), "   c    c\n");
    assert_eq!(instance.unexpand_line("        c"), "\tc\n");
    assert_eq!(instance.unexpand_line("        c        c"), "\tc\tc\n");

    let mut instance = Unexpand { all: true, tabs: TabStops::new(Some("2,+4")).unwrap() };
    assert_eq!(instance.unexpand_line("  c"), "\tc\n");
    assert_eq!(instance.unexpand_line("          c"), "\t\t\tc\n");
    assert_eq!(instance.unexpand_line("  c    c"), "\tc\tc\n");
    assert_eq!(instance.unexpand_line("   c    c"), "\t c\tc\n");
    assert_eq!(instance.unexpand_line("    c    c"), "\t  c\tc\n");
    assert_eq!(instance.unexpand_line("      c    c"), "\t\tc\tc\n");
    assert_eq!(instance.unexpand_line("      c        c"), "\t\tc\t\tc\n");
    assert_eq!(instance.unexpand_line("      c        c    "), "\t\tc\t\tc\t\n");

    let mut instance = Unexpand { all: true, tabs: TabStops::new(Some("2,/4")).unwrap() };
    assert_eq!(instance.unexpand_line("  c"), String::from("\tc\n"));
    assert_eq!(instance.unexpand_line("    c    c"), "\t\tc\tc\n");

    let mut instance = Unexpand { all: true, tabs: TabStops::new(Some("2 4 6")).unwrap() };
    assert_eq!(instance.unexpand_line("      c"), "\t\t\tc\n");

    // backspace tests
    let mut instance = Unexpand { all: true, tabs: TabStops::new(Some("2")).unwrap() };
    assert_eq!(instance.unexpand_line("     c"), "\t\t c\n");
    assert_eq!(instance.unexpand_line("     c"), "\t\t c\n");
}

#[test]
fn new() {
    let instance = TabStops::new(Some("2")).unwrap();
    assert_eq!(instance.offset, None);
    assert_eq!(instance.positions, vec![]);
    assert_eq!(instance.repetable, Some(2));

    let instance = TabStops::new(Some("4")).unwrap();
    assert_eq!(instance.repetable, Some(4));

    let instance = TabStops::new(Some("1,2")).unwrap();
    assert_eq!(instance.offset, None);
    assert_eq!(instance.repetable, None);
    assert_eq!(instance.positions, vec![1, 2]);
}

#[test]
fn new_values_with_repetable() {
    let instance = TabStops::new(Some("1,2,/4")).unwrap();
    assert_eq!(instance.offset, None);
    assert_eq!(instance.repetable, Some(4));
    assert_eq!(instance.positions, (vec![1, 2]));
}

#[test]
fn new_values_with_prefix() {
    let instance = TabStops::new(Some("1,+8")).unwrap();
    assert_eq!(instance.offset, Some(1));
    assert_eq!(instance.repetable, Some(8));
    assert_eq!(instance.positions, vec![]);

    let instance = TabStops::new(Some("1,2,+4")).unwrap();
    assert_eq!(instance.offset, Some(2));
    assert_eq!(instance.repetable, Some(4));
    assert_eq!(instance.positions, (vec![1]));
}

#[test]
#[should_panic(expected = "unexpand: tab sizes must be ascending")]
fn new_panic_ascending() { TabStops::new(Some("2,1")).unwrap(); }

#[test]
#[should_panic(expected = "unexpand: tab sizes must be ascending")]
fn new_panic_ascending2() { TabStops::new(Some("2,2")).unwrap(); }

#[test]
#[should_panic(expected = "unexpand: tab size cannot be 0")]
fn new_panic_zero() { TabStops::new(Some("0")).unwrap(); }

#[test]
#[should_panic(expected = "unexpand: tab size cannot be 0")]
fn new_panic_zero_values() { TabStops::new(Some("0,1")).unwrap(); }

#[test]
#[should_panic(expected = "unexpand: error parsing arguments")]
fn new_panic_wrong_type() { TabStops::new(Some("a")).unwrap(); }

#[test]
#[should_panic(expected = "unexpand: error parsing arguments")]
fn new_panic_wrong_type_multipe_with_prefix() { TabStops::new(Some("a, +b")).unwrap(); }

#[test]
#[should_panic(expected = "unexpand: error parsing arguments")]
fn new_panic_wrong_type_multipe() { TabStops::new(Some("a, b")).unwrap(); }

#[test]
fn is_tab_stop_repetable() {
    let instance = TabStops::new(Some("2")).unwrap();

    for i in 1..50 {
        assert_eq!(instance.is_tab_stop(i), i % 2 == 0);
    }
}

#[test]
fn is_tab_stop_values() {
    let instance = TabStops::new(Some("1, 2, 4")).unwrap();
    assert_eq!(instance.is_tab_stop(1), true);
    assert_eq!(instance.is_tab_stop(2), true);
    assert_eq!(instance.is_tab_stop(3), false);
    assert_eq!(instance.is_tab_stop(4), true);

    for i in 5..50 {
        assert_eq!(instance.is_tab_stop(i), false);
    }
}

#[test]
fn is_tab_stop_with_offset() {
    let instance = TabStops::new(Some("1,+8")).unwrap();
    assert_eq!(instance.is_tab_stop(1), true);
    assert_eq!(instance.is_tab_stop(2), false);
    assert_eq!(instance.is_tab_stop(8), false);
    assert_eq!(instance.is_tab_stop(9), true);
    assert_eq!(instance.is_tab_stop(16), false);
    assert_eq!(instance.is_tab_stop(17), true);
    assert_eq!(instance.is_tab_stop(18), false);
}

#[test]
fn is_tab_stop_with_values_and_repetable() {
    let instance = TabStops::new(Some("1, 2, /8")).unwrap();
    assert_eq!(instance.is_tab_stop(1), true);
    assert_eq!(instance.is_tab_stop(2), true);
    assert_eq!(instance.is_tab_stop(3), false);
    assert_eq!(instance.is_tab_stop(8), true);
    assert_eq!(instance.is_tab_stop(9), false);
    assert_eq!(instance.is_tab_stop(16), true);
}

#[test]
fn tab_stops_separate_by_blanks() {
    let instance = TabStops::new(Some("2 4")).unwrap();
    assert_eq!(instance.positions, vec![2, 4]);
}
