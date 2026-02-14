/// Wraps a section with start/end markers to help the model parse context.
pub fn cover(title: &str, content: &str) -> String {
    let t = title.to_uppercase();
    format!(":: START {t} ::\n{content}\n:: END {t} ::")
}

/// Comments every line, used to show the convention under the result.
pub fn comment(text: &str) -> String {
    text.lines()
        .map(|line| format!("# {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Ensures there is a blank line between title and body.
pub fn normalize_commit_message(message: &str) -> String {
    let trimmed = message.trim_end();
    let lines: Vec<&str> = trimmed.split('\n').collect();
    if lines.len() <= 1 {
        return trimmed.to_string();
    }
    if !lines[1].is_empty() {
        let mut out = String::new();
        out.push_str(lines[0]);
        out.push('\n');
        out.push('\n');
        out.push_str(&lines[1..].join("\n"));
        return out;
    }
    trimmed.to_string()
}
