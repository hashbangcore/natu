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
use std::process::Command;

#[derive(Clone)]
struct CommandCompleter {
    commands: Vec<&'static str>,
}

impl Helper for CommandCompleter {}
impl Hinter for CommandCompleter {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for CommandCompleter {}

impl Validator for CommandCompleter {}

impl Completer for CommandCompleter {
    type Candidate = Pair;

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

fn create_prompt(history: &str, user_input: &str, command_output: Option<&str>) -> String {
    let command_section = match command_output {
        Some(output) => format!(
            "
:: COMMAND OUTPUT (SYSTEM) ::

{}

:: END COMMAND OUTPUT (SYSTEM) ::
",
            output
        ),
        None => String::new(),
    };

    format!(
        "
LLM ROL: Conversational terminal assistant
USERNAME: {}
DATETIME: {}


:: INSTRUCTION (SYSTEM) ::

- Keep responses concise: 5-20 lines maximum.
- Do not use emojis or decorations.
- Always prioritize the latest user message over the HISTORICAL CHAT. 
- The latest message may be completely unrelated to previous messages. 
- Do not assume continuity or context from 
  the history unless the user explicitly refers to it.

:: END INSTRUCTION (SYSTEM) ::

:: HISTORIAL CHAT (SYSTEM) ::

{}

:: END HISTORIAL CHAT (SYSTEM) ::

{}

:: USER MESSAGE ::

{}

:: END USER MESSAGE ::
",
        utils::get_user(),
        utils::current_datetime(),
        history,
        command_section,
        user_input
    )
}

fn extract_inline_commands(input: &str) -> Vec<String> {
    let bytes = input.as_bytes();
    let mut commands = Vec::new();
    let mut i = 0;

    while i + 1 < bytes.len() {
        if bytes[i] == b'$' && bytes[i + 1] == b'(' {
            let mut j = i + 2;
            let mut depth = 1;

            while j < bytes.len() {
                if bytes[j] == b'(' {
                    depth += 1;
                } else if bytes[j] == b')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += 1;
            }

            if depth == 0 {
                let cmd = input[i + 2..j].trim().to_string();
                if !cmd.is_empty() {
                    commands.push(cmd);
                }
                i = j + 1;
                continue;
            } else {
                break;
            }
        }
        i += 1;
    }

    commands
}

fn strip_inline_commands(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'(' {
            let mut j = i + 2;
            let mut depth = 1;

            while j < bytes.len() {
                if bytes[j] == b'(' {
                    depth += 1;
                } else if bytes[j] == b')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += 1;
            }

            if depth == 0 {
                i = j + 1;
                continue;
            }
        }

        output.push(bytes[i] as char);
        i += 1;
    }

    output.trim().to_string()
}

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

#[derive(Debug)]
enum EvalError {
    Empty,
    InvalidToken(char),
    MismatchedParens,
    DivisionByZero,
}

fn format_eval_error(err: EvalError) -> String {
    match err {
        EvalError::Empty => "expresión vacía".to_string(),
        EvalError::InvalidToken(ch) => format!("token inválido: '{}'", ch),
        EvalError::MismatchedParens => "paréntesis desbalanceados".to_string(),
        EvalError::DivisionByZero => "división por cero".to_string(),
    }
}

fn eval_expr(input: &str) -> Result<i64, EvalError> {
    let mut parser = Parser::new(input);
    let value = parser.parse_expr()?;
    parser.skip_ws();
    if parser.peek().is_some() {
        return Err(EvalError::InvalidToken(parser.peek().unwrap()));
    }
    Ok(value)
}

struct Parser<'a> {
    iter: std::str::Chars<'a>,
    lookahead: Option<char>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        let mut iter = input.chars();
        let lookahead = iter.next();
        Self { iter, lookahead }
    }

    fn peek(&self) -> Option<char> {
        self.lookahead
    }

    fn next(&mut self) -> Option<char> {
        let current = self.lookahead;
        self.lookahead = self.iter.next();
        current
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_whitespace()) {
            self.next();
        }
    }

    fn parse_expr(&mut self) -> Result<i64, EvalError> {
        self.skip_ws();
        let mut value = self.parse_term()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('+') => {
                    self.next();
                    value = value + self.parse_term()?;
                }
                Some('-') => {
                    self.next();
                    value = value - self.parse_term()?;
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn parse_term(&mut self) -> Result<i64, EvalError> {
        self.skip_ws();
        let mut value = self.parse_factor()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('*') => {
                    self.next();
                    value = value * self.parse_factor()?;
                }
                Some('/') => {
                    self.next();
                    let rhs = self.parse_factor()?;
                    if rhs == 0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    value = value / rhs;
                }
                Some('%') => {
                    self.next();
                    let rhs = self.parse_factor()?;
                    if rhs == 0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    value = value % rhs;
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn parse_factor(&mut self) -> Result<i64, EvalError> {
        self.skip_ws();
        match self.peek() {
            Some('-') => {
                self.next();
                Ok(-self.parse_factor()?)
            }
            Some('(') => {
                self.next();
                let value = self.parse_expr()?;
                self.skip_ws();
                match self.next() {
                    Some(')') => Ok(value),
                    _ => Err(EvalError::MismatchedParens),
                }
            }
            Some(ch) if ch.is_ascii_digit() => self.parse_number(),
            Some(ch) => Err(EvalError::InvalidToken(ch)),
            None => Err(EvalError::Empty),
        }
    }

    fn parse_number(&mut self) -> Result<i64, EvalError> {
        self.skip_ws();
        let mut buf = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                buf.push(ch);
                self.next();
            } else {
                break;
            }
        }
        if buf.is_empty() {
            return Err(EvalError::Empty);
        }
        buf.parse::<i64>()
            .map_err(|_| EvalError::InvalidToken(buf.chars().next().unwrap()))
    }
}

pub async fn connect(service: &core::Service, args: &core::Cli) {
    let mut history: Vec<String> = Vec::new();
    let mut rl = Editor::<CommandCompleter, DefaultHistory>::new()
        .expect("failed to initialize rustyline editor");
    rl.set_helper(Some(CommandCompleter {
        commands: vec!["/clean", "/trans", "/eval", "/help"],
    }));

    loop {
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
        let prompt = create_prompt(&dialog, &cleaned_input, command_output.as_deref());

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
