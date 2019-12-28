use clap::{load_yaml, App, AppSettings::ColoredHelp, ArgMatches};

fn main() {
    let yaml = load_yaml!("template.yml");
    let _matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();
}
