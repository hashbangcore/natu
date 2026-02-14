use crate::core;

use super::format::{comment, cover, normalize_commit_message};
use super::git::staged_changes;
use super::prompts::{convention, instruction, skeleton};

fn generate(hint: Option<&str>) -> String {
    let user_hint = hint.unwrap_or("");

    let context = "repository context";
    let staged_changes = staged_changes();

    let sections = [
        ("INSTRUCTION", instruction().to_string()),
        ("CONVENTION", convention().to_string()),
        ("SKELETON", skeleton().to_string()),
        ("PROJECT CONTEXT", context.to_string()),
        ("USER HINT", user_hint.to_string()),
        ("REPOSITORY STATUS", staged_changes),
    ];

    sections
        .iter()
        .map(|(title, content)| cover(title, content))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Builds a commit prompt, calls the model, and prints the final message.
pub async fn generate_commit(
    service: &core::Service,
    args: &core::Cli,
    hint: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let prompt = generate(hint);

    if args.verbose {
        println!("{}\n\n", prompt);
    }

    let result = service.complete(&prompt).await?;
    let result = normalize_commit_message(&result);

    // TODO: manejar de forma más segura
    match result.contains("Error: no changes staged for commit") {
        true => println!("{}", result),
        false => println!("{}\n\n\n{}", result.trim_end(), comment(convention())),
    }

    Ok(())
}
