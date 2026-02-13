use crate::core;
use crate::utils;
use rustyline::Context;
use rustyline::Editor;
use rustyline::Helper;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use std::fs::File;
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
    let mut rl = Editor::<CommandCompleter, DefaultHistory>::new()
        .expect("failed to initialize rustyline editor");
    rl.set_helper(Some(CommandCompleter {
        commands: vec!["/clean", "/trans", "/eval", "/help"],
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
            continue;
        }

        if user_input == "/help" {
            println!(
                "\nCommands:\n\
/help  Show this help message\n\
/clean Clear chat history\n\
/trans Translate text (uses LLM)\n\
/eval  Evaluate arithmetic expression\n"
            );
            continue;
        }

        if let Some(rest) = user_input.strip_prefix("/trans") {
            let text = strip_inline_commands(rest).trim().to_string();
            if text.is_empty() {
                continue;
            }

            let prompt = format!(
                "
Task: Translate the following text faithfully, preserving its meaning and context.
Do not explain or add anything. Return only the translation.When 
processing text, recognize and handle the lang:lang syntax (e.g., :en for English) 
as a directive for language specification. 
Ensure responses adhere to the language indicated by the directive. 
If no directive is provided, default to the user's preferred language . 
Do not interpret lang:lang as literal text.\n\nTEXT:\n{}",
                text
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

        match service.complete(&prompt).await {
            Ok(text) => {
                let output = utils::render_markdown(&text);
                println!("\n{}", output);

                history.push(format!("{}: {}", utils::get_user(), cleaned_input));
                history.push(format!("Assistant: {}\n", text));
            }
            Err(err) => {
                eprintln!("AI error: {}", err);
                break;
            }
        }
    }
}
