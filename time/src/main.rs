mod cli;

fn main() {
    let _ = cli::create_app().get_matches();
}
