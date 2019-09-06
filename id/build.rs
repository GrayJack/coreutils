use std::env;

use clap::{ App, Shell, load_yaml };

fn main() {
    let yaml = load_yaml!("src/id.yml");
    let mut app = App::from_yaml(yaml);

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        _ => return
    };

    app.gen_completions("id", Shell::Zsh, out_dir.clone());
    app.gen_completions("id", Shell::Fish, out_dir.clone());
    app.gen_completions("id", Shell::Bash, out_dir.clone());
    app.gen_completions("id", Shell::PowerShell, out_dir.clone());
    app.gen_completions("id", Shell::Elvish, out_dir);
}
