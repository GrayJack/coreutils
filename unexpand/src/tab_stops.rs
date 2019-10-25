#[derive(Debug)]
pub struct TabStops {
    offset:    Option<usize>,
    repetable: Option<usize>,
    positions: Vec<usize>,
}

const ARG_PARSE_MSG: &str = "unexpand: error parsing arguments";

impl TabStops {
    pub fn new(tabs_str: Option<&str>) -> Result<Self, &str> {
        match tabs_str {
            Some(tabs_str) => {
                if tabs_str == "" {
                    return Ok(TabStops { offset: None, repetable: Some(8), positions: vec![] });
                }

                let tabs_str = tabs_str.replace(", ", ",");
                let mut tabs_vec: Vec<&str> =
                    tabs_str.split(|c| c == ',' || c == ' ').map(str::trim).collect();

                if tabs_vec.len() == 1 {
                    let value = tabs_vec[0].parse::<usize>().map_err(|_err| ARG_PARSE_MSG)?;

                    if value == 0 {
                        return Err("unexpand: tab size cannot be 0");
                    }

                    return Ok(TabStops {
                        offset:    None,
                        repetable: Some(value),
                        positions: vec![],
                    });
                }

                let mut offset: Option<usize> = None;
                let mut repetable: Option<usize> = None;
                let last_item = String::from(*tabs_vec.last().unwrap());

                if last_item.contains('+') {
                    repetable = Some(
                        tabs_vec.pop().unwrap()[1..]
                            .parse::<usize>()
                            .map_err(|_err| ARG_PARSE_MSG)?,
                    );
                    offset = Some(
                        tabs_vec.pop().unwrap().parse::<usize>().map_err(|_err| ARG_PARSE_MSG)?,
                    );
                }

                if last_item.contains('/') {
                    repetable =
                        Some(last_item[1..].parse::<usize>().map_err(|_err| ARG_PARSE_MSG)?);
                    tabs_vec.pop();
                }

                let mut positions: Vec<usize> = vec![];
                for tab_val in &tabs_vec {
                    positions.push(tab_val.parse::<usize>().map_err(|_err| ARG_PARSE_MSG)?);
                }

                if !positions.is_empty() {
                    if positions.contains(&0) {
                        return Err("unexpand: tab size cannot be 0");
                    }

                    for i in 0..(positions.len() - 1) {
                        if positions[i + 1] <= positions[i] {
                            return Err("unexpand: tab sizes must be ascending");
                        }
                    }

                    return Ok(TabStops { offset, repetable, positions });
                }

                Ok(TabStops { offset, repetable, positions: vec![] })
            },
            None => Ok(TabStops { offset: None, repetable: Some(8), positions: vec![] }),
        }
    }

    pub fn is_tab_stop(self: &Self, column: usize) -> bool {
        if self.positions.contains(&column) {
            return true;
        }

        if self.repetable.is_some() && column % self.repetable.unwrap() == self.offset.unwrap_or(0)
        {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
