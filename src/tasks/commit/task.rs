use crate::core;

use std::process::Command;

fn prompt_instruction() -> &'static str {
    include_str!("prompts/instruction.txt")
}

fn prompt_convention() -> &'static str {
    include_str!("prompts/convention.txt")
}

fn prompt_skeleton() -> &'static str {
    include_str!("prompts/skeleton.txt")
}

fn cover(title: &str, content: &str) -> String {
    let t = title.to_uppercase();
    format!(":: START {t} ::\n{content}\n:: END {t} ::")
}

fn comment(text: &str) -> String {
    text.lines()
        .map(|line| format!("# {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn staged_changes() -> String {
    // TODO: implementar la funcionalidad de larry
    let output = Command::new("larry")
        .args([
            "git branch -v",
            "git remote -v",
            "git log --stat -1",
            "echo '*GIT LOG*' && git log --oneline -10",
            "git status -s",
            "git status",
            "git diff --cached --quiet && echo 'No staged changes' || echo 'Staged changes present' && git diff --staged",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => String::from_utf8(out.stdout)
            .unwrap_or_else(|e| format!("UTF-8 error in git diff --staged: {e}")),
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            format!("git diff --staged failed:\n{err}")
        }
        Err(e) => format!("Failed to execute git diff --staged: {e}"),
    }
}

fn generate(hint: Option<&str>) -> String {
    let user_hint = hint.unwrap_or("");

    let context = "repository context";
    let staged_changes = staged_changes();

    let sections = [
        ("INSTRUCTION", prompt_instruction().to_string()),
        ("CONVENTION", prompt_convention().to_string()),
        ("SKELETON", prompt_skeleton().to_string()),
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

pub async fn generate_commit(
    service: &core::Service,
    args: &core::interfaz::Cli,
    hint: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let prompt = generate(hint);

    if args.verbose {
        println!("{}\n\n", prompt);
    }

    let result = service.complete(&prompt).await?;

    // TODO: manejar de forma más segura
    match result.contains("Error: no changes staged for commit") {
        true => println!("{}", result),
        false => println!("{}\n\n\n{}", result, comment(prompt_convention())),
    }

    Ok(())
}
