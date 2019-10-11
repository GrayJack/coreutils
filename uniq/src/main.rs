use clap::{load_yaml, App};

mod execution_options;

fn main() {
    let yaml = load_yaml!("uniq.yml");
    let _opts = read_clap_matches(App::from_yaml(yaml).get_matches());
}
