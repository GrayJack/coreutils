use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("nohup.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let string = if matches.is_present("STRING") {
        let inputs = matches.values_of("STRING").unwrap();
        inputs.fold(String::new(), |res, s| res + s + " ")
    } else {
        "y".to_string()
    };

    loop {
        println!("{}", string);
    }
}
