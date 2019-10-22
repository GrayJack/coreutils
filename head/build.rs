use std::env;

use clap::{load_yaml, App, Shell};

fn main() {
    let yaml = load_yaml!("src/head.yml");
    let mut app = App::from_yaml(yaml);

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        _ => return,
    };

    app.gen_completions("head", Shell::Zsh, out_dir.clone());
    app.gen_completions("head", Shell::Fish, out_dir.clone());
    app.gen_completions("head", Shell::Bash, out_dir.clone());
    app.gen_completions("head", Shell::PowerShell, out_dir.clone());
    app.gen_completions("head", Shell::Elvish, out_dir);
}
