#[derive(Debug)]
pub struct TabStops {
    offset: Option<usize>,
    repetable: Option<usize>,
    positions: Vec<usize>,
}

impl TabStops {
    pub fn new(tabs_str: Option<&str>) -> TabStops {
        match tabs_str {
            Some(tabs_str) => {
                if tabs_str == "" {
                    return TabStops { offset: None, repetable: Some(8), positions: vec![] };
                }

                let mut tabs_vec: Vec<&str> = tabs_str.split(',').map(|s| s.trim()).collect();

                if tabs_vec.len() == 1 {
                    let value = tabs_vec[0].parse::<usize>().unwrap();

                    if value == 0 {
                        panic!("unexpand: tab size cannot be 0");
                    }

                    return TabStops { offset: None, repetable: Some(value), positions: vec![] };
                }

                let mut offset: Option<usize> = None;
                let mut repetable: Option<usize> = None;
                let last_item = tabs_vec.last().unwrap().clone();

                if last_item.contains(&"+") {
                    repetable = tabs_vec.pop().unwrap()[1..].parse::<usize>().ok();
                    offset = tabs_vec.pop().unwrap().parse::<usize>().ok();
                }

                if last_item.contains(&"/") {
                    repetable = last_item[1..].parse::<usize>().ok();
                    tabs_vec.pop();
                }

                let positions: Vec<usize> =
                    tabs_vec.iter().map(|p| p.parse::<usize>().unwrap()).collect();

                if positions.len() > 0 {
                    if positions.contains(&0) {
                        panic!("unexpand: tab size cannot be 0");
                    }

                    for i in 0..(positions.len() - 1) {
                        if positions[i + 1] <= positions[i] {
                            panic!("unexpand: tab sizes must be ascending");
                        }
                    }

                    return TabStops { offset, repetable, positions: positions };
                }

                TabStops { offset, repetable, positions: vec![] }
            }
            None => TabStops { offset: None, repetable: Some(8), positions: vec![] },
        }
    }

    pub fn is_tab_stop(self: &Self, column: usize) -> bool {
        if self.positions.contains(&column) {
            return true;
        }

        if self.repetable.is_some() && column % self.repetable.unwrap() == self.offset.unwrap_or(0) {
            return true;
        }

        false
    }
}

#[test]
fn new() {
    let instance = TabStops::new(Some("2"));
    assert_eq!(instance.offset, None);
    assert_eq!(instance.positions, vec![]);
    assert_eq!(instance.repetable, Some(2));

    let instance = TabStops::new(Some("4"));
    assert_eq!(instance.repetable, Some(4));

    let instance = TabStops::new(Some("1,2"));
    assert_eq!(instance.offset, None);
    assert_eq!(instance.repetable, None);
    assert_eq!(instance.positions, vec![1, 2]);
}

#[test]
fn new_values_with_repetable() {
    let instance = TabStops::new(Some("1,2,/4"));
    assert_eq!(instance.offset, None);
    assert_eq!(instance.repetable, Some(4));
    assert_eq!(instance.positions, (vec![1, 2]));
}

#[test]
fn new_values_with_prefix() {
    let instance = TabStops::new(Some("1,+8"));
    assert_eq!(instance.offset, Some(1));
    assert_eq!(instance.repetable, Some(8));
    assert_eq!(instance.positions, vec![]);

    let instance = TabStops::new(Some("1,2,+4"));
    assert_eq!(instance.offset, Some(2));
    assert_eq!(instance.repetable, Some(4));
    assert_eq!(instance.positions, (vec![1]));
}

#[test]
#[should_panic(expected = "unexpand: tab sizes must be ascending")]
fn new_panic_ascending() {
    TabStops::new(Some("2,1"));
}

#[test]
#[should_panic(expected = "unexpand: tab sizes must be ascending")]
fn new_panic_ascending2() {
    TabStops::new(Some("2,2"));
}

#[test]
#[should_panic(expected = "unexpand: tab size cannot be 0")]
fn new_panic_zero() {
    TabStops::new(Some("0"));
}

#[test]
#[should_panic(expected = "unexpand: tab size cannot be 0")]
fn new_panic_zero_values() {
    TabStops::new(Some("0,1"));
}


#[test]
fn is_tab_stop_repetable() {
    let instance = TabStops::new(Some("2"));

    for i in 1..50 {
        assert_eq!(instance.is_tab_stop(i), i % 2 == 0);
    }
}

#[test]
fn is_tab_stop_values() {
    let instance = TabStops::new(Some("1, 2, 4"));
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
    let instance = TabStops::new(Some("1,+8"));
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
    let instance = TabStops::new(Some("1, 2, /8"));
    assert_eq!(instance.is_tab_stop(1), true);
    assert_eq!(instance.is_tab_stop(2), true);
    assert_eq!(instance.is_tab_stop(3), false);
    assert_eq!(instance.is_tab_stop(8), true);
    assert_eq!(instance.is_tab_stop(9), false);
    assert_eq!(instance.is_tab_stop(16), true);
}
