use std::env;

use clap::{load_yaml, App, Shell};

fn main() {
    let yaml = load_yaml!("src/wc.yml");
    let mut app = App::from_yaml(yaml);

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        _ => return,
    };

    app.gen_completions("wc", Shell::Zsh, out_dir.clone());
    app.gen_completions("wc", Shell::Fish, out_dir.clone());
    app.gen_completions("wc", Shell::Bash, out_dir.clone());
    app.gen_completions("wc", Shell::PowerShell, out_dir.clone());
    app.gen_completions("wc", Shell::Elvish, out_dir);
}
