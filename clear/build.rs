use std::env;

use clap::{load_yaml, App, Shell};

fn main() {
    let yaml = load_yaml!("src/clear.yml");
    let mut app = App::from_yaml(yaml);

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        _ => return,
    };

    app.gen_completions("clear", Shell::Zsh, out_dir.clone());
    app.gen_completions("clear", Shell::Fish, out_dir.clone());
    app.gen_completions("clear", Shell::Bash, out_dir.clone());
    app.gen_completions("clear", Shell::PowerShell, out_dir.clone());
    app.gen_completions("clear", Shell::Elvish, out_dir);
}
