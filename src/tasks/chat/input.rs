use rustyline::Editor;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use super::commands::CommandCompleter;

/// Initializes the line editor with command completion.
pub fn new_editor() -> Editor<CommandCompleter, DefaultHistory> {
    let mut rl = Editor::<CommandCompleter, DefaultHistory>::new()
        .expect("failed to initialize rustyline editor");
    rl.set_helper(Some(CommandCompleter::new(vec![
        "/clean", "/trans", "/eval", "/help", "/stream", "/add",
    ])));
    rl
}

/// Opens a TTY reader when stdin is piped, so we can still read user input.
pub fn open_tty_reader(stdin_is_piped: bool) -> Result<Option<BufReader<File>>, String> {
    if !stdin_is_piped {
        return Ok(None);
    }
    File::open("/dev/tty")
        .map(BufReader::new)
        .map(Some)
        .map_err(|err| format!("Error: {}", err))
}

/// Reads one line of user input from TTY or rustyline.
pub fn read_user_input(
    rl: &mut Editor<CommandCompleter, DefaultHistory>,
    tty_reader: &mut Option<BufReader<File>>,
) -> Result<Option<String>, String> {
    if let Some(reader) = tty_reader.as_mut() {
        let mut stdout = std::io::stdout();
        stdout
            .write_all(b"\x1b[36m\xE2\x9E\x9C ")
            .map_err(|_| "Error writing prompt".to_string())?;
        stdout
            .flush()
            .map_err(|_| "Error flushing prompt".to_string())?;
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => Ok(None),
            Ok(_) => {
                stdout
                    .write_all(b"\x1b[0m")
                    .map_err(|_| "Error resetting color".to_string())?;
                Ok(Some(line.trim().to_string()))
            }
            Err(err) => Err(format!("Error: {}", err)),
        }
    } else {
        println!("\x1b[36m");
        let readline = rl.readline("➜ ");
        let user_input = match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())
                    .map_err(|_| "Error adding history".to_string())?;
                line.trim().to_string()
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => return Ok(None),
            Err(err) => return Err(format!("Error: {:?}", err)),
        };
        println!("\x1b[0m");
        Ok(Some(user_input))
    }
}
