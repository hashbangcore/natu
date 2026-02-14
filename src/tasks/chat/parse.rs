/// Extracts inline command substitutions of the form `#!(...)` from a user input string.
pub fn extract_inline_commands(input: &str) -> Vec<String> {
    let bytes = input.as_bytes();
    let mut commands = Vec::new();
    let mut i = 0;

    while i + 2 < bytes.len() {
        if bytes[i] == b'#' && bytes[i + 1] == b'!' && bytes[i + 2] == b'(' {
            let mut j = i + 3;
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
                let cmd = input[i + 3..j].trim().to_string();
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

/// Removes inline command substitutions of the form `#!(...)` from the input and trims the result.
pub fn strip_inline_commands(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if i + 2 < bytes.len() && bytes[i] == b'#' && bytes[i + 1] == b'!' && bytes[i + 2] == b'(' {
            let mut j = i + 3;
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

/// Splits a command line into arguments, honoring single and double quotes.
pub fn split_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    for ch in input.chars() {
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
