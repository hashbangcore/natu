use crate::core;
use crate::tasks::render;
use crate::utils;
use rustyline::Context;
use rustyline::Helper;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use std::fs;
use std::io::Write;

use super::eval::{eval_expr, format_eval_error};
use super::lang::{lang_display_name, normalize_lang_tag};
use super::parse::{split_args, strip_inline_commands};

const HELP_TEXT: &str = "\nCommands:\n\
/help  Show this help message\n\
/clean Clear chat history\n\
/add   Attach file contents to chat context\n\
/trans Translate text (uses LLM)\n\
/eval  Evaluate arithmetic expression\n\
/save  Save an informe about the chat\n\
/stream [on|off] Toggle streaming output\n";

/// Provides command name completions for slash-prefixed commands in the prompt.
pub struct CommandCompleter {
    /// The set of slash commands available for completion.
    commands: Vec<&'static str>,
    /// Filename completer for /add paths.
    file_completer: FilenameCompleter,
}

impl CommandCompleter {
    pub fn new(commands: Vec<&'static str>) -> Self {
        Self {
            commands,
            file_completer: FilenameCompleter::new(),
        }
    }
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
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        if line.starts_with("/add ") && pos >= 5 {
            return self.file_completer.complete(line, pos, ctx);
        }
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

pub fn handle_help(user_input: &str) -> bool {
    if user_input == "/help" {
        println!("{HELP_TEXT}");
        return true;
    }
    false
}

pub fn handle_clean(user_input: &str, history: &mut Vec<String>) -> bool {
    if user_input == "/clean" {
        history.clear();
        print!("\x1b[2J\x1b[H");
        let _ = std::io::stdout().flush();
        return true;
    }
    false
}

pub fn handle_stream(user_input: &str, stream_enabled: &mut bool) -> bool {
    let Some(rest) = user_input.strip_prefix("/stream") else {
        return false;
    };
    let mode = rest.trim().to_lowercase();
    if mode == "on" {
        *stream_enabled = true;
        println!("\nstream: on");
    } else if mode == "off" {
        *stream_enabled = false;
        println!("\nstream: off");
    } else {
        println!("\nUsage: /stream on|off");
    }
    true
}

pub fn handle_add(
    user_input: &str,
    history: &mut Vec<String>,
    pending_stdin: &mut Option<String>,
) -> bool {
    let Some(rest) = user_input.strip_prefix("/add") else {
        return false;
    };
    let args = split_args(rest.trim());
    if args.is_empty() {
        println!("\nUsage: /add <path> [path2 path3 ...]");
        return true;
    }

    let mut attachment = String::new();
    for path in args {
        match fs::read_to_string(&path) {
            Ok(content) => {
                attachment.push_str("\n-- FILE: ");
                attachment.push_str(&path);
                attachment.push_str(" --\n");
                attachment.push_str(&content);
                attachment.push('\n');
                history.push(format!("Attachment: {}\n{}\n", path, content));
                println!("\nadded: {}", path);
            }
            Err(err) => {
                eprintln!("\nError reading {}: {}", path, err);
            }
        }
    }

    if !attachment.is_empty() {
        *pending_stdin = Some(attachment);
    }
    true
}

pub fn handle_eval(user_input: &str) -> bool {
    let Some(rest) = user_input.strip_prefix("/eval") else {
        return false;
    };
    let expr = strip_inline_commands(rest).trim().to_string();
    if expr.is_empty() {
        println!("\nUsage: /eval <expression>");
        return true;
    }

    match eval_expr(&expr) {
        Ok(value) => println!("\n{}", value),
        Err(err) => println!("\nError: {}", format_eval_error(err)),
    }

    true
}

pub async fn handle_trans(
    user_input: &str,
    service: &core::Service,
    args: &core::Cli,
) -> Result<bool, String> {
    let Some(rest) = user_input.strip_prefix("/trans") else {
        return Ok(false);
    };
    let raw_text = strip_inline_commands(rest).trim().to_string();
    if raw_text.is_empty() {
        println!("\nUsage: /trans [INPUT_LANG:OUTPUT_LANG] <text>");
        return Ok(true);
    }

    let (input_lang, output_lang, text) = parse_lang_directive(&raw_text);
    if text.is_empty() {
        return Ok(true);
    }

    let user_lang = normalize_lang_tag(&utils::get_user_lang());
    let target_lang = output_lang
        .as_deref()
        .map(normalize_lang_tag)
        .unwrap_or(user_lang);
    let source_lang = input_lang
        .as_deref()
        .map(normalize_lang_tag)
        .unwrap_or_else(|| "auto-detect".to_string());
    let target_lang_name = lang_display_name(&target_lang);

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
            let output = render::render_markdown(&text);
            println!("\n{}", output);
            Ok(true)
        }
        Err(err) => Err(format!("AI error: {}", err)),
    }
}

pub async fn handle_save(
    user_input: &str,
    service: &core::Service,
    args: &core::Cli,
    history: &[String],
) -> Result<bool, String> {
    let Some(rest) = user_input.strip_prefix("/save") else {
        return Ok(false);
    };
    let raw_text = strip_inline_commands(rest).trim().to_string();

    let datetime = utils::current_datetime();
    let user_lang = utils::get_user_lang();
    let history_text = history.join("\n");
    let prompt = if raw_text.is_empty() {
        format!(
            "Write an informe for the user.\n\
Use the same language as the user.\n\
User language: {user_lang}\n\
Do not add footers, notes, or meta commentary.\n\
Chat history:\n\
{history_text}\n"
        )
    } else {
        format!(
            "Hint (required): {raw_text}\n\
Chat history:\n\
{history_text}\n"
        )
    };

    if args.verbose {
        println!("\x1b[32m{}\x1b[0m", prompt);
    }

    let result = match service.complete(&prompt).await {
        Ok(text) => text,
        Err(err) => return Err(format!("AI error: {}", err)),
    };

    let output = result.trim_end().to_string();
    let safe_datetime = datetime.replace(' ', ".").replace(':', "_");
    let filename = format!("netero.{}.md", safe_datetime);
    let path = filename.as_str();
    let write_result = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| std::io::Write::write_all(&mut file, output.as_bytes()));

    match write_result {
        Ok(()) => {
            println!("\nsaved: {}", path);
            Ok(true)
        }
        Err(err) => Err(format!("File error: {}", err)),
    }
}

fn parse_lang_directive(raw_text: &str) -> (Option<String>, Option<String>, &str) {
    let mut input_lang: Option<String> = None;
    let mut output_lang: Option<String> = None;
    let mut text = raw_text;

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

    (input_lang, output_lang, text)
}
