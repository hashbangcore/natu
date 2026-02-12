use crate::core;
use crate::utils;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;

fn create_prompt(history: &str, user_input: &str) -> String {
    format!(
        "
LLM: Netero
LLM ROL: Terminal tool
USER: {}
USER LANG: ES
DISTRO: NixOS
DATETIME: {}

:: INSTRUCTION (SYSTEM) ::

- Do not take any proactive actions, 
  including polite questions, unless 
  explicitly requested by the user.
- Do not include farewell messages in any response.
- Do not use emojis.
- Keep responses brief (5-20 lines maximum),
  keep answers concise, avoid long explanations.

:: END INSTRUCTION (SYSTEM)::


:: COMMANDS (SYSTEM) ::

/t, translate [source_lang]:[target_lang] [text] 
/e, eval [expresion] = [true | false]

:: END COMMANDS (SYSTEM) ::


:: HISTORIAL CHAT (SYSTEM) ::

{}

:: END HISTORIAL CHAT (SYSTEM) ::


:: USER MESSAGE ::

{}

:: END USER MESSAGE ::
",
        utils::get_user(),
        utils::current_datetime(),
        history,
        user_input
    )
}

pub async fn connect(service: &core::Service, args: &core::Cli) {
    let provider = args.provider.clone();
    //let provider = "codestral".to_string();
    let mut history: Vec<String> = Vec::new();
    let mut rl =
        Editor::<(), DefaultHistory>::new().expect("failed to initialize rustyline editor");

    println!("::: {} :::\n", provider);

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

        let dialog = history.join("\n");
        let prompt = create_prompt(&dialog, &user_input);

        if args.verbose {
            println!("\x1b[32m{}\x1b[0m", prompt);
        }

        match service.complete(&prompt).await {
            Ok(text) => {
                let output = utils::render_markdown(&text);
                println!("\n{}", output);

                history.push(format!("{}: {}", utils::get_user(), user_input));
                history.push(format!("Assistant: {}\n", text));
            }
            Err(err) => {
                eprintln!("AI error: {}", err);
                break;
            }
        }
    }
}
