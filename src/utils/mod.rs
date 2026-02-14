use chrono::Local;
use std::env;
use std::io::{self, IsTerminal, Read};
use termimad::MadSkin;

pub fn get_stdin() -> String {
    let mut input = String::new();

    if !io::stdin().is_terminal() {
        io::stdin().read_to_string(&mut input).unwrap();
    }
    return input;
}

pub fn stdin_is_piped() -> bool {
    !io::stdin().is_terminal()
}

pub fn get_user() -> String {
    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    capitalize(&user)
}

pub fn get_user_lang() -> String {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    "unknown".to_string()
}

pub fn current_datetime() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn capitalize(s: &str) -> String {
    s.get(0..1).unwrap_or("").to_uppercase() + s.get(1..).unwrap_or("")
}

pub fn render_markdown(response: &str) -> String {
    let skin = MadSkin::default();
    skin.term_text(response).to_string()
}
