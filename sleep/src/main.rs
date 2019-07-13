use std::process;

use clap::{load_yaml, App};
use sugars::sleep;

fn main() {
    let yaml = load_yaml!("sleep.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let numbers = match matches.values_of("NUMBER") {
        Some(values) => values,
        None => {
            eprintln!("Missing operand.\nTry 'sleep --help' for more information.");
            process::exit(1);
        }
    };

    let total: u64 = numbers.filter_map(|s| s.parse::<u64>().ok()).sum();

    let suffix = if matches.is_present("suffix") {
        matches.value_of("suffix").unwrap()
    } else {
        "s"
    };

    match suffix {
        "s" | "sec" => sleep!(total sec),
        "m" | "min" => sleep!(total min),
        "h" | "hour" => {
            let total = 60 * total;
            sleep!(total min)
        }
        _ => {
            eprintln!("Invalid suffix value. It must be 'sec', 'min', 'hour', 's', 'm' or 'h'.\nFor more information, try 'sleep --help'.");
            process::exit(1);
        }
    }
}
