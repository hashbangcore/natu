use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "netero",
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

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Open a minimal chat session
    Chat,

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

    /// Generate shell completion
    Completion { shell: clap_complete::Shell },
}
