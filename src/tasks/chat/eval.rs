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

    fn parse_expr(&mut self) -> Result<i64, EvalError> {
        let mut value = self.parse_term()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('+') => {
                    self.next();
                    value = value
                        .checked_add(self.parse_term()?)
                        .ok_or(EvalError::InvalidToken('+'))?;
                }
                Some('-') => {
                    self.next();
                    value = value
                        .checked_sub(self.parse_term()?)
                        .ok_or(EvalError::InvalidToken('-'))?;
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn parse_term(&mut self) -> Result<i64, EvalError> {
        let mut value = self.parse_factor()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('*') => {
                    self.next();
                    value = value
                        .checked_mul(self.parse_factor()?)
                        .ok_or(EvalError::InvalidToken('*'))?;
                }
                Some('/') => {
                    self.next();
                    let rhs = self.parse_factor()?;
                    if rhs == 0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    value /= rhs;
                }
                Some('%') => {
                    self.next();
                    let rhs = self.parse_factor()?;
                    if rhs == 0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    value %= rhs;
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn parse_factor(&mut self) -> Result<i64, EvalError> {
        self.skip_ws();
        match self.peek() {
            Some('(') => {
                self.next();
                let value = self.parse_expr()?;
                self.skip_ws();
                if self.next() != Some(')') {
                    return Err(EvalError::MismatchedParens);
                }
                Ok(value)
            }
            Some('-') => {
                self.next();
                Ok(-self.parse_factor()?)
            }
            Some(ch) if ch.is_ascii_digit() => self.parse_number(),
            Some(ch) => Err(EvalError::InvalidToken(ch)),
            None => Err(EvalError::Empty),
        }
    }

    fn parse_number(&mut self) -> Result<i64, EvalError> {
        let mut num = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num.push(ch);
                self.next();
            } else {
                break;
            }
        }
        num.parse::<i64>().map_err(|_| EvalError::Empty)
    }
}
