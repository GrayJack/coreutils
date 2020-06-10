mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

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
