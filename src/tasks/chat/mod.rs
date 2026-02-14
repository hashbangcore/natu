mod commands;
mod eval;
mod inline_exec;
mod input;
mod lang;
mod parse;
mod prompt;
mod stream;
pub mod task;

pub use task::generate_chat as connect;
