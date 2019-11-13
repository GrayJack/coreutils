use std::env;

use clap::{load_yaml, App, Shell};

fn main() {
    let yaml = load_yaml!("src/nohup.yml");
    let mut app = App::from_yaml(yaml)
        .help_message("Display help information")
        .version_message("Display version information");

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        _ => return,
    };

    app.gen_completions("nohup", Shell::Zsh, out_dir.clone());
    app.gen_completions("nohup", Shell::Fish, out_dir.clone());
    app.gen_completions("nohup", Shell::Bash, out_dir.clone());
    app.gen_completions("nohup", Shell::PowerShell, out_dir.clone());
    app.gen_completions("nohup", Shell::Elvish, out_dir);
}
