mod core;
mod tasks;
mod utils;

use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;
use tasks::chat;
use tasks::commit;
use tasks::message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = utils::get_stdin();
    let args = core::Cli::parse();
    let service = core::Service::new(Some(&args.provider));

    execute(&service, &args, stdin).await?;

    Ok(())
}

async fn execute(
    service: &core::Service,
    args: &core::Cli,
    stdin: String,
) -> Result<(), Box<dyn std::error::Error>> {
    match &args.command {
        Some(core::Commands::Commit { hint }) => {
            commit::generate(service, args, hint.as_deref()).await?
        }
        Some(core::Commands::Prompt { input }) => {
            message::generate(service, args, &input, stdin).await?
        }
        Some(core::Commands::Chat) => chat::generate(service, args).await,
        Some(core::Commands::Completion { shell }) => {
            let mut cmd = core::Cli::command();
            generate(*shell, &mut cmd, "netero", &mut std::io::stdout());
        }
        None => {
            if let Some(prompt) = args.prompt.as_deref() {
                message::generate(service, args, prompt, stdin).await?;
            } else {
                chat::generate(service, args).await;
            }
        }
    }

    Ok(())
}
