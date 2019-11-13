use std::env;

use clap::{load_yaml, App, Shell};

fn main() {
    let yaml = load_yaml!("src/dirname.yml");
    let mut app = App::from_yaml(yaml)
        .help_message("Display help information")
        .version_message("Display version information");

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        _ => return,
    };

    app.gen_completions("dirname", Shell::Zsh, out_dir.clone());
    app.gen_completions("dirname", Shell::Fish, out_dir.clone());
    app.gen_completions("dirname", Shell::Bash, out_dir.clone());
    app.gen_completions("dirname", Shell::PowerShell, out_dir.clone());
    app.gen_completions("dirname", Shell::Elvish, out_dir);
}
