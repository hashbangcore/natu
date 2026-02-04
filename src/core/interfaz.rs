use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "rave",
    author,
    version,
    about = "CLI for interacting with language models",
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Prompt passed to the language model
    pub prompt: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Language model provider
    #[arg(long, short, default_value = "codestral")]
    pub provider: String,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate a commit message
    Commit {
        /// Optional prompt used as commit context
        hint: Option<String>,
    },

    /// Process a single prompt
    Prompt {
        /// Prompt provided via the command line
        input: String,
    },
}
