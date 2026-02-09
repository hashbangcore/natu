mod core;
mod tasks;
mod utils;

use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;
use core::interfaz;
use tasks::chat;
use tasks::commit;
use tasks::message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = utils::get_stdin();
    let args = interfaz::Cli::parse();
    let service = core::Service::new(Some(&args.provider));

    execute(&service, &args, stdin).await?;

    Ok(())
}

async fn execute(
    service: &core::Service,
    args: &core::interfaz::Cli,
    stdin: String,
) -> Result<(), Box<dyn std::error::Error>> {
    match &args.command {
        Some(interfaz::Commands::Commit { hint }) => {
            commit::generate(service, args, hint.as_deref()).await?
        }
        Some(interfaz::Commands::Prompt { input }) => {
            message::generate(service, args, &input, stdin).await?
        }
        Some(interfaz::Commands::Chat) => chat::generate(service, args).await,
        Some(interfaz::Commands::Completion { shell }) => {
            let mut cmd = interfaz::Cli::command();
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
