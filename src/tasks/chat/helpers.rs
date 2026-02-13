/// Builds the chat prompt body from already-resolved user, datetime, history, and input values.
pub fn create_prompt(
    username: &str,
    datetime: &str,
    history: &str,
    user_input: &str,
    command_output: Option<&str>,
    stdin_attachment: Option<&str>,
) -> String {
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

    let stdin_section = match stdin_attachment {
        Some(content) => format!(
            "
:: STDIN ATTACHMENT (SYSTEM) ::

{}

:: END STDIN ATTACHMENT (SYSTEM) ::
",
            content
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

{}

:: USER MESSAGE ::

{}

:: END USER MESSAGE ::
",
        username, datetime, history, command_section, stdin_section, user_input
    )
}

/// Extracts inline command substitutions of the form `$(...)` from a user input string.
pub fn extract_inline_commands(input: &str) -> Vec<String> {
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

/// Removes inline command substitutions of the form `$(...)` from the input and trims the result.
pub fn strip_inline_commands(input: &str) -> String {
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

/// Represents arithmetic parsing failures for the `/eval` command.
#[derive(Debug)]
pub enum EvalError {
    /// The expression is empty or contains no numeric tokens.
    Empty,
    /// The expression contains a token that cannot be parsed.
    InvalidToken(char),
    /// Parentheses are unbalanced or missing.
    MismatchedParens,
    /// Division or modulo by zero was attempted.
    DivisionByZero,
}

/// Formats arithmetic parsing failures into user-facing messages.
pub fn format_eval_error(err: EvalError) -> String {
    match err {
        EvalError::Empty => "expresión vacía".to_string(),
        EvalError::InvalidToken(ch) => format!("token inválido: '{}'", ch),
        EvalError::MismatchedParens => "paréntesis desbalanceados".to_string(),
        EvalError::DivisionByZero => "división por cero".to_string(),
    }
}

/// Evaluates a simple arithmetic expression with +, -, *, /, %, and parentheses.
pub fn eval_expr(input: &str) -> Result<i64, EvalError> {
    let mut parser = ArithmeticExpressionParser::new(input);
    let value = parser.parse_expr()?;
    parser.skip_ws();
    if parser.peek().is_some() {
        return Err(EvalError::InvalidToken(parser.peek().unwrap()));
    }
    Ok(value)
}

/// Parses arithmetic expressions using a recursive-descent parser.
struct ArithmeticExpressionParser<'a> {
    /// Iterator for walking the input string.
    iter: std::str::Chars<'a>,
    /// Single-character lookahead used by the parser.
    lookahead: Option<char>,
}

impl<'a> ArithmeticExpressionParser<'a> {
    /// Creates a new parser over the provided input string.
    fn new(input: &'a str) -> Self {
        let mut iter = input.chars();
        let lookahead = iter.next();
        Self { iter, lookahead }
    }

    /// Peeks at the next character without consuming it.
    fn peek(&self) -> Option<char> {
        self.lookahead
    }

    /// Consumes and returns the next character.
    fn next(&mut self) -> Option<char> {
        let current = self.lookahead;
        self.lookahead = self.iter.next();
        current
    }

    /// Skips any whitespace characters.
    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_whitespace()) {
            self.next();
        }
    }

    /// Parses an expression containing `+` and `-` operators.
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

    /// Parses a term containing `*`, `/`, and `%` operators.
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

    /// Parses a numeric literal, a parenthesized expression, or a unary `-`.
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

    /// Parses a sequence of digits into an integer value.
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
