use std::io::{stdout, Write};

fn main() {
    let stdout = stdout();
    let _ = stdout.lock().write(b"\x1b[2J\x1b[H");
}
