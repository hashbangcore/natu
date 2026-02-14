/// Builds the chat prompt body from already-resolved user, datetime, history, and input values.
pub fn create_prompt(
    username: &str,
    datetime: &str,
    user_lang: &str,
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

    // NOTE: user_lang should reflect the OS locale (e.g., LANG/LC_ALL).
    format!(
        "
LLM ROL: Conversational terminal assistant
USERNAME: {}
DATETIME: {}
USER LANG: {}

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
        username, datetime, user_lang, history, command_section, stdin_section, user_input
    )
}
