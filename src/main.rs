mod core;
mod task;

use chrono::Local;
use clap::Parser;
use core::interfaz;
use std::env;
use std::io::{self, IsTerminal, Read};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin = String::new();

    if !io::stdin().is_terminal() {
        io::stdin().read_to_string(&mut stdin).unwrap();
    }

    let args = interfaz::Cli::parse();
    let ai = core::Service::new(Some(&args.provider));

    let ctx = core::CliContext {
        ai,
        stdin,
        verbose: args.verbose,
        provider: args.provider.to_string(),
    };

    execute(&ctx, args).await?;

    Ok(())
}

async fn execute(
    ctx: &core::CliContext,
    args: interfaz::Cli,
) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        Some(interfaz::Commands::Commit { hint }) => generate_commit(ctx, hint.as_deref()).await?,
        Some(interfaz::Commands::Prompt { input }) => send_chat(ctx, &input).await?,
        None => {
            if let Some(prompt) = args.prompt {
                send_chat(ctx, &prompt).await?;
            } else {
                eprintln!("Error: a message is required for chat or commit");
            }
        }
    }

    Ok(())
}

async fn generate_commit(
    ctx: &core::CliContext,
    hint: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let prompt = task::commit::prompt::generate(hint);

    if ctx.verbose {
        println!("{}\n\n", prompt);
    }

    let result = ctx.ai.complete(&prompt).await?;

    println!("{}", result);

    Ok(())
}

fn capitalize(s: &str) -> String {
    s.get(0..1).unwrap_or("").to_uppercase() + s.get(1..).unwrap_or("")
}

fn current_datetime() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

async fn send_chat(
    ctx: &core::CliContext,
    request: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    let preamble = format!(
        "LLM name: Netero\nUser name: {}\nDate and hour: {}\n",
        capitalize(&user),
        current_datetime()
    );

    let prompt = if ctx.stdin.trim().is_empty() {
        format!("User request:\n {}\n", request.trim())
    } else {
        format!(
            "== USER REQUEST ==\n{}\n== END USER REQUEST ==\n\n== STDIN FILE ==\n{}\n== END STDIN FILE ==\n",
            request.trim(),
            ctx.stdin.trim()
        )
    };

    let wrapper = format!("{}\n{}", preamble, prompt);

    let response = ctx.ai.complete(&wrapper).await?;

    if ctx.verbose {
        println!("\x1b[1m{}:\x1b[0m\n\n{}\n", user.to_uppercase(), wrapper);
        println!(
            "\x1b[1m{}:\x1b[0m\n\n{}",
            ctx.provider.to_uppercase(),
            response.trim()
        );
    } else {
        println!("{}", response.trim());
    }

    Ok(())
}
