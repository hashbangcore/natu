use std::process::Command;

/// Collects git status and staged diff to give context to the model.
pub fn staged_changes() -> String {
    run_commands(&[
        "git status -sb",
        "git diff --cached --quiet && echo 'No staged changes' || (git diff --staged --stat --no-color && git diff --staged --no-color)",
    ])
}

/// Runs a list of shell commands and returns a structured report.
pub fn run_commands(commands: &[&str]) -> String {
    let mut sections = Vec::with_capacity(commands.len());

    for cmd_str in commands {
        let output = Command::new("sh").arg("-c").arg(cmd_str).output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let combined_output = if !stderr.is_empty() {
                    format!("{}{}", stdout, stderr)
                } else {
                    stdout.to_string()
                };

                sections.push(format!(
                    "[section]\n[command]\n{}\n[output]\n{}\n[end section]",
                    cmd_str,
                    combined_output.trim_end()
                ));
            }
            Err(err) => {
                sections.push(format!(
                    "[section]\n[command]\n{}\n[error]\n{}\n[end section]",
                    cmd_str, err
                ));
            }
        }
    }

    sections.join("\n\n")
}
