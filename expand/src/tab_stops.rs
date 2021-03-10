#[derive(Debug)]
pub struct TabStops {
    pub(crate) offset: Option<usize>,
    pub(crate) repetable: Option<usize>,
    pub(crate) positions: Vec<usize>,
}

const ARG_PARSE_MSG: &str = "expand: error parsing arguments";

impl TabStops {
    pub fn new(tabs_str: Option<&str>) -> Result<Self, String> {
        match tabs_str {
            Some(tabs_str) => {
                if tabs_str.is_empty() {
                    return Ok(TabStops { offset: None, repetable: Some(8), positions: vec![] });
                }

                let tabs_str = tabs_str.replace(", ", ",");
                let mut tabs_vec: Vec<&str> =
                    tabs_str.split(|c| c == ',' || c == ' ').map(str::trim).collect();

                if tabs_vec.len() == 1 {
                    let value = tabs_vec[0]
                        .parse::<usize>()
                        .map_err(|err| format!("{}: {}", ARG_PARSE_MSG, err))?;

                    if value == 0 {
                        return Err("expand: tab size cannot be 0".to_string());
                    }

                    return Ok(TabStops {
                        offset: None,
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
                            .map_err(|err| format!("{}: {}", ARG_PARSE_MSG, err))?,
                    );
                    offset = Some(
                        tabs_vec
                            .pop()
                            .unwrap()
                            .parse::<usize>()
                            .map_err(|err| format!("{}: {}", ARG_PARSE_MSG, err))?,
                    );
                }

                if last_item.contains('/') {
                    repetable = Some(
                        last_item[1..]
                            .parse::<usize>()
                            .map_err(|err| format!("{}: {}", ARG_PARSE_MSG, err))?,
                    );
                    tabs_vec.pop();
                }

                let mut positions: Vec<usize> = vec![];
                for tab_val in &tabs_vec {
                    positions.push(
                        tab_val
                            .parse::<usize>()
                            .map_err(|err| format!("{}: {}", ARG_PARSE_MSG, err))?,
                    );
                }

                if !positions.is_empty() {
                    if positions.contains(&0) {
                        return Err("expand: tab size cannot be 0".to_string());
                    }

                    for i in 0..(positions.len() - 1) {
                        if positions[i + 1] <= positions[i] {
                            return Err("expand: tab sizes must be ascending".to_string());
                        }
                    }

                    return Ok(TabStops { offset, repetable, positions });
                }

                Ok(TabStops { offset, repetable, positions: vec![] })
            },
            None => Ok(TabStops { offset: None, repetable: Some(8), positions: vec![] }),
        }
    }
}
