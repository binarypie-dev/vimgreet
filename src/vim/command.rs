use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Reboot,
    Poweroff,
    Session(Option<String>),
    User(Option<String>),
    Login,
    Cancel,
    Help,
    Quit,
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Unknown command: {0}")]
    Unknown(String),
}

pub fn parse_command(input: &str) -> Result<Command, CommandError> {
    let input = input.trim();
    let mut parts = input.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let arg = parts.next().map(|s| s.trim().to_string());

    match cmd {
        "reboot" | "rb" => Ok(Command::Reboot),
        "poweroff" | "shutdown" | "po" => Ok(Command::Poweroff),
        "session" | "s" => Ok(Command::Session(arg)),
        "user" | "u" => Ok(Command::User(arg)),
        "login" | "l" => Ok(Command::Login),
        "cancel" | "c" => Ok(Command::Cancel),
        "help" | "h" | "?" => Ok(Command::Help),
        "q" | "quit" | "exit" => Ok(Command::Quit),
        "" => Err(CommandError::Unknown("empty command".to_string())),
        other => Err(CommandError::Unknown(other.to_string())),
    }
}

