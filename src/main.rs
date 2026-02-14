mod core;
mod tasks;
mod utils;

use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;
use tasks::chat;
use tasks::commit;
use tasks::pipeline;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin_is_piped = utils::stdin_is_piped();
    let stdin = utils::get_stdin();
    let args = core::Cli::parse();

    if args.trace && (args.command.is_some() || !args.prompt.is_empty()) {
        let mut cmd = core::Cli::command();
        cmd.error(
            clap::error::ErrorKind::ArgumentConflict,
            "--trace cannot be used with subcommands or prompt input",
        )
        .exit();
    }

    if args.trace {
        core::trace::run_trace_server().await?;
        return Ok(());
    }

    let service = core::Service::new(&args);

    execute(&service, &args, stdin, stdin_is_piped).await?;

    Ok(())
}

async fn execute(
    service: &core::Service,
    args: &core::Cli,
    stdin: String,
    stdin_is_piped: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match &args.command {
        Some(core::Commands::Commit { hint, convention }) => {
            let hint_text = if hint.is_empty() {
                None
            } else {
                Some(hint.join(" "))
            };
            commit::connect(service, args, hint_text.as_deref(), convention.as_deref()).await?
        }
        Some(core::Commands::Prompt { input }) => {
            let input_text = input.join(" ");
            pipeline::connect(service, args, &input_text, stdin).await?
        }
        Some(core::Commands::Chat) => chat::connect(service, args, stdin, stdin_is_piped).await,
        Some(core::Commands::Completion { shell }) => {
            let mut cmd = core::Cli::command();
            generate(*shell, &mut cmd, "netero", &mut std::io::stdout());
        }
        None => {
            if args.prompt.is_empty() {
                chat::connect(service, args, stdin, stdin_is_piped).await;
            } else {
                let prompt_text = args.prompt.join(" ");
                pipeline::connect(service, args, &prompt_text, stdin).await?;
            }
        }
    }

    Ok(())
}
