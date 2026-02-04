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
    format!("== START {t} ==\n{content}\n== END {t} ==")
}

fn staged_changes() -> String {
    let output = Command::new("git").args(["diff", "--staged"]).output();

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

pub fn generate(hint: Option<&str>) -> String {
    let user_hint = hint.unwrap_or("");

    let context = "repository context";
    let staged_changes = staged_changes();

    let sections = [
        ("INSTRUCTION", prompt_instruction().to_string()),
        ("CONVENTION", prompt_convention().to_string()),
        ("SKELETON", prompt_skeleton().to_string()),
        ("PROJECT CONTEXT", context.to_string()),
        ("USER HINT", user_hint.to_string()),
        ("STAGED CHANGES", staged_changes),
    ];

    sections
        .iter()
        .map(|(title, content)| cover(title, content))
        .collect::<Vec<_>>()
        .join("\n\n")
}
