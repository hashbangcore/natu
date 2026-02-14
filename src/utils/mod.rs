#![allow(unused)]

pub mod env;
pub mod io;
pub mod lang;
pub mod strings;
pub mod time;

pub use env::{get_user, get_user_lang};
pub use io::{get_stdin, stdin_is_piped};
pub use lang::normalize_lang_tag;
pub use strings::capitalize;
pub use time::current_datetime;
