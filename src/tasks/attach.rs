use std::fs;
use std::env;

/// File attachment extracted from user input.
pub struct Attachment {
    /// Path as written by the user (not expanded).
    pub path: String,
    pub content: String,
}

/// Returns true if the token looks like a file path.
fn is_path_candidate(token: &str) -> bool {
    token.starts_with('/')
        || token.starts_with("./")
        || token.starts_with("../")
        || token.starts_with("~/")
}

fn expand_path(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            return format!("{}/{}", home, rest);
        }
    }
    path.to_string()
}

fn read_file(path: &str) -> Option<String> {
    fs::read_to_string(path).ok()
}

/// Splits input into tokens, honoring quotes and backslash escapes.
pub fn split_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        if ch == '\\' && quote != Some('\'') {
            escape = true;
            continue;
        }

        if let Some(q) = quote {
            if ch == q {
                quote = None;
            } else {
                current.push(ch);
            }
            continue;
        }

        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }

        if ch.is_whitespace() {
            if !current.is_empty() {
                args.push(current.clone());
                current.clear();
            }
            continue;
        }

        current.push(ch);
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

/// Extracts file attachments from tokenized input.
pub fn extract_attachments_from_tokens(
    tokens: &[String],
) -> (Vec<String>, Vec<Attachment>) {
    let mut remaining = Vec::new();
    let mut attachments = Vec::new();

    for token in tokens {
        if !is_path_candidate(token) {
            remaining.push(token.clone());
            continue;
        }

        let expanded = expand_path(token);
        match fs::metadata(&expanded) {
            Ok(meta) if meta.is_file() => {
                if let Some(content) = read_file(&expanded) {
                    attachments.push(Attachment {
                        path: token.clone(),
                        content,
                    });
                } else {
                    remaining.push(token.clone());
                }
            }
            _ => remaining.push(token.clone()),
        }
    }

    (remaining, attachments)
}

/// Extracts file attachments from a raw input string.
pub fn extract_attachments_from_input(input: &str) -> (String, Vec<Attachment>) {
    let tokens = split_args(input);
    let (_remaining, attachments) = extract_attachments_from_tokens(&tokens);
    (input.to_string(), attachments)
}

/// Formats attachments into a single block, compatible with stdin attachments.
pub fn format_attachments(attachments: &[Attachment]) -> Option<String> {
    if attachments.is_empty() {
        return None;
    }
    let mut out = String::new();
    for attachment in attachments {
        out.push_str("\n-- FILE: ");
        out.push_str(&attachment.path);
        out.push_str(" --\n");
        out.push_str(&attachment.content);
        out.push('\n');
    }
    Some(out)
}

fn indent_block(content: &str, prefix: &str) -> String {
    if content.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let mut lines = content.lines().peekable();
    while let Some(line) = lines.next() {
        out.push_str(prefix);
        out.push_str(line);
        if lines.peek().is_some() {
            out.push('\n');
        }
    }
    if content.ends_with('\n') {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(prefix);
    }
    out
}

/// Formats stdin and file attachments into a single attached files block.
pub fn format_attached_files(
    stdin: Option<&str>,
    attachments: &[Attachment],
) -> Option<String> {
    let mut sections = Vec::new();
    if let Some(content) = stdin {
        if !content.trim().is_empty() {
            sections.push(format!(
                "-- FILE: STDIN --\n{}",
                indent_block(content, "      ")
            ));
        }
    }
    for attachment in attachments {
        sections.push(format!(
            "-- FILE: {} --\n{}",
            attachment.path,
            indent_block(&attachment.content, "      ")
        ));
    }
    if sections.is_empty() {
        return None;
    }
    let mut out = String::new();
    out.push_str(":: ATTACHED FILES ::\n\n");
    out.push_str(&sections.join("\n\n"));
    out.push_str("\n\n:: END ATTACHED FILES ::");
    Some(out)
}
