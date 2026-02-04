use std::io;
use std::sync::OnceLock;

pub static STDIN: OnceLock<String> = OnceLock::new();

pub fn start() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();

    STDIN.set(buffer.trim().to_string()).ok();
}
