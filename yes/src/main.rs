use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("yes.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

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
