use std::io::{self, BufWriter, Write};

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let string = if matches.is_present("STRING") {
        let inputs = matches.values_of("STRING").unwrap();
        inputs.fold(String::new(), |res, s| res + s + " ")
    } else {
        "y".to_string()
    };

    let mut stdout = BufWriter::new(io::stdout());

    loop {
        writeln!(stdout, "{}", string).unwrap_or_else(|err| {
            eprintln!("yes: failed to write to standard out: {}", err);
            std::process::exit(1);
        });
    }
}
