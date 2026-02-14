use crate::core;
use crate::utils;
use futures_util::StreamExt;
use rustyline::Context;
use rustyline::Editor;
use rustyline::Helper;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use serde_json::json;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::process::Command;

use super::helpers::create_prompt;
use super::helpers::eval_expr;
use super::helpers::extract_inline_commands;
use super::helpers::format_eval_error;
use super::helpers::strip_inline_commands;

#[derive(Clone)]
/// Provides command name completions for slash-prefixed commands in the prompt.
struct CommandCompleter {
    /// The set of slash commands available for completion.
    commands: Vec<&'static str>,
}

/// Enables rustyline helper integration for slash command completion.
impl Helper for CommandCompleter {}
/// Disables hints while still fulfilling the rustyline helper contract.
impl Hinter for CommandCompleter {
    type Hint = String;

    /// Returns no hint so user input remains unobstructed.
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

/// Disables highlighting while still fulfilling the rustyline helper contract.
impl Highlighter for CommandCompleter {}

/// Disables validation while still fulfilling the rustyline helper contract.
impl Validator for CommandCompleter {}

/// Implements slash command completion for rustyline.
impl Completer for CommandCompleter {
    type Candidate = Pair;

    /// Returns completions when the current token starts with `/`.
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let start = line[..pos]
            .rfind(|c: char| c.is_whitespace())
            .map(|idx| idx + 1)
            .unwrap_or(0);
        let prefix = &line[start..pos];

        if !prefix.starts_with('/') {
            return Ok((pos, Vec::new()));
        }

        let matches = self
            .commands
            .iter()
            .filter(|cmd| cmd.starts_with(prefix))
            .map(|cmd| Pair {
                display: cmd.to_string(),
                replacement: cmd.to_string(),
            })
            .collect();

        Ok((start, matches))
    }
}

/// Executes inline shell commands and returns a formatted output section, if any.
fn run_inline_commands(user_input: &str) -> Option<String> {
    let commands = extract_inline_commands(user_input);
    if commands.is_empty() {
        return None;
    }

    let mut entries = Vec::new();

    for cmd in commands {
        let output = Command::new("bash").args(["-lc", &cmd]).output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).trim_end().to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).trim_end().to_string();

                if out.status.success() {
                    let stdout_display = if stdout.is_empty() {
                        "<empty>"
                    } else {
                        &stdout
                    };
                    entries.push(format!(
                        "[section]\n[command]\n{}\n\n[stdout]\n{}\n[end section]",
                        cmd, stdout_display
                    ));
                    if !stderr.is_empty() {
                        entries.push(format!("[stderr]\n{}", stderr));
                    }
                } else {
                    let stderr_display = if stderr.is_empty() {
                        "<empty>"
                    } else {
                        &stderr
                    };
                    let stdout_display = if stdout.is_empty() {
                        "<empty>"
                    } else {
                        &stdout
                    };
                    entries.push(format!(
                        "$({})\n[exit status]\n{}\n[stderr]\n{}\n[stdout]\n{}",
                        cmd, out.status, stderr_display, stdout_display
                    ));
                }
            }
            Err(err) => {
                entries.push(format!("$({})\n[error]\n{}", cmd, err));
            }
        }
    }

    Some(entries.join("\n\n"))
}

async fn stream_completion(
    service: &core::Service,
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let body = json!({
        "model": service.model,
        "messages": [
            { "role": "user", "content": prompt }
        ],
        "stream": true
    });

    let mut req = service.http.post(&service.endpoint).json(&body);

    if let Some(key) = &service.apikey {
        req = req.header("Authorization", format!("Bearer {}", key));
    }

    let response = req.send().await?;
    let mut stream = response.bytes_stream();
    let mut content = String::new();
    let mut stdout = std::io::stdout();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        let text = String::from_utf8_lossy(&chunk);

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            let data = data.trim();
            if data == "[DONE]" {
                stdout.write_all(b"\n")?;
                stdout.flush()?;
                return Ok(content);
            }
            let parsed: serde_json::Value = serde_json::from_str(data)?;
            let delta = parsed["choices"][0]["delta"]["content"]
                .as_str()
                .unwrap_or("");
            if !delta.is_empty() {
                content.push_str(delta);
                stdout.write_all(delta.as_bytes())?;
                stdout.flush()?;
            }
        }
    }

    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(content)
}

/// Starts the interactive chat session and handles all supported commands.
pub async fn generate_chat(
    service: &core::Service,
    args: &core::Cli,
    stdin: String,
    stdin_is_piped: bool,
) {
    let mut history: Vec<String> = Vec::new();
    let mut pending_stdin = if stdin.trim().is_empty() {
        None
    } else {
        Some(stdin)
    };
    let mut stream_enabled = false;
    let mut rl = Editor::<CommandCompleter, DefaultHistory>::new()
        .expect("failed to initialize rustyline editor");
    rl.set_helper(Some(CommandCompleter {
        commands: vec!["/clean", "/trans", "/eval", "/help", "/stream", "/add"],
    }));
    let mut tty_reader = if stdin_is_piped {
        match File::open("/dev/tty") {
            Ok(file) => Some(BufReader::new(file)),
            Err(err) => {
                eprintln!("Error: {}", err);
                return;
            }
        }
    } else {
        None
    };

    loop {
        let user_input = if let Some(reader) = tty_reader.as_mut() {
            let mut stdout = std::io::stdout();
            if stdout
                .write_all(b"\x1b[36m\xE2\x9E\x9C ")
                .is_err()
            {
                break;
            }
            if stdout.flush().is_err() {
                break;
            }
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if stdout.write_all(b"\x1b[0m").is_err() {
                        break;
                    }
                    line.trim().to_string()
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    break;
                }
            }
        } else {
            println!("\x1b[36m");
            let readline = rl.readline("➜ ");
            let user_input = match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str()).unwrap();
                    line.trim().to_string()
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            };
            println!("\x1b[0m");
            user_input
        };

        if user_input.is_empty() {
            continue;
        }

        if user_input == "/clean" {
            history.clear();
            print!("\x1b[2J\x1b[H");
            let _ = std::io::stdout().flush();
            continue;
        }

        if user_input == "/help" {
            println!(
                "\nCommands:\n\
/help  Show this help message\n\
/clean Clear chat history\n\
/add   Attach file contents to chat context\n\
/trans Translate text (uses LLM)\n\
/eval  Evaluate arithmetic expression\n\
/stream [on|off] Toggle streaming output\n"
            );
            continue;
        }

        if let Some(rest) = user_input.strip_prefix("/add") {
            let mut path = rest.trim();
            if (path.starts_with('"') && path.ends_with('"'))
                || (path.starts_with('\'') && path.ends_with('\''))
            {
                path = &path[1..path.len().saturating_sub(1)];
            }

            if path.is_empty() {
                println!("\nUsage: /add <path>");
                continue;
            }

            match fs::read_to_string(path) {
                Ok(content) => {
                    pending_stdin = Some(content.clone());
                    history.push(format!("Attachment: {}\n", content));
                    println!("\nadded: {}", path);
                }
                Err(err) => {
                    eprintln!("\nError reading {}: {}", path, err);
                }
            }
            continue;
        }

        if let Some(rest) = user_input.strip_prefix("/stream") {
            let mode = rest.trim().to_lowercase();
            if mode == "on" {
                stream_enabled = true;
                println!("\nstream: on");
            } else if mode == "off" {
                stream_enabled = false;
                println!("\nstream: off");
            } else {
                println!("\nUsage: /stream on|off");
            }
            continue;
        }

        if let Some(rest) = user_input.strip_prefix("/trans") {
            let raw_text = strip_inline_commands(rest).trim().to_string();
            if raw_text.is_empty() {
                continue;
            }

            let mut input_lang: Option<String> = None;
            let mut output_lang: Option<String> = None;
            let mut text = raw_text.as_str();

            let mut parts = raw_text.splitn(2, char::is_whitespace);
            let first = parts.next().unwrap_or("");
            if first.contains(':')
                && first
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == ':')
            {
                let mut lang_parts = first.splitn(2, ':');
                let in_lang = lang_parts.next().unwrap_or("").trim();
                let out_lang = lang_parts.next().unwrap_or("").trim();
                if !in_lang.is_empty() {
                    input_lang = Some(in_lang.to_string());
                }
                if !out_lang.is_empty() {
                    output_lang = Some(out_lang.to_string());
                }
                if input_lang.is_some() || output_lang.is_some() {
                    text = parts.next().unwrap_or("").trim();
                }
            }

            if text.is_empty() {
                continue;
            }

            let user_lang = utils::normalize_lang_tag(&utils::get_user_lang());
            let target_lang = output_lang
                .as_deref()
                .map(utils::normalize_lang_tag)
                .unwrap_or(user_lang);
            let source_lang = input_lang
                .as_deref()
                .map(utils::normalize_lang_tag)
                .unwrap_or_else(|| "auto-detect".to_string());
            let target_lang_name = utils::lang_display_name(&target_lang);

            let prompt = format!(
                "
Task: Translate the following text faithfully, preserving its meaning and context.
Return only the translation. Do not explain or add anything.
You must translate. Do not choose any other task or language.
LANG: {}:{}.
Source language (locked): {}.
Target language (locked): {}.
Target language name (locked): {}.

TEXT:
{}",
                source_lang, target_lang, source_lang, target_lang, target_lang_name, text
            );

            if args.verbose {
                println!("\x1b[32m{}\x1b[0m", prompt);
            }

            match service.complete(&prompt).await {
                Ok(text) => {
                    let output = utils::render_markdown(&text);
                    println!("\n{}", output);
                }
                Err(err) => {
                    eprintln!("AI error: {}", err);
                    break;
                }
            }

            continue;
        }

        if let Some(rest) = user_input.strip_prefix("/eval") {
            let expr = strip_inline_commands(rest).trim().to_string();
            if expr.is_empty() {
                continue;
            }

            match eval_expr(&expr) {
                Ok(value) => println!("\n{}", value),
                Err(err) => println!("\nError: {}", format_eval_error(err)),
            }

            continue;
        }

        let dialog = history.join("\n");
        let command_output = run_inline_commands(&user_input);
        let cleaned_input = strip_inline_commands(&user_input);
        let prompt = create_prompt(
            &utils::get_user(),
            &utils::current_datetime(),
            &utils::get_user_lang(),
            &dialog,
            &cleaned_input,
            command_output.as_deref(),
            pending_stdin.as_deref(),
        );
        if pending_stdin.is_some() {
            pending_stdin = None;
        }

        if args.verbose {
            println!("\x1b[32m{}\x1b[0m", prompt);
        }

        let response = if stream_enabled {
            match stream_completion(service, &prompt).await {
                Ok(text) => text,
                Err(err) => {
                    eprintln!("AI error: {}", err);
                    break;
                }
            }
        } else {
            match service.complete(&prompt).await {
                Ok(text) => {
                    let output = utils::render_markdown(&text);
                    println!("\n{}", output);
                    text
                }
                Err(err) => {
                    eprintln!("AI error: {}", err);
                    break;
                }
            }
        };

        history.push(format!("{}: {}", utils::get_user(), cleaned_input));
        history.push(format!("Assistant: {}\n", response));
    }
}
